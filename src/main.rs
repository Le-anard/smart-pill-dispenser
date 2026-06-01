mod wifi;
mod display;
mod servo;
mod buzzer;
mod led;
mod thingsboard;
mod scheduler;

use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::gpio::{PinDriver, Pull, InterruptType};
use esp_idf_hal::task::notification::Notification;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::thread;
use std::time::Duration;
use chrono::{Local, Datelike}; 
use core::num::NonZeroU32;
use log::info;

use scheduler::MedicationSchedule;
use thingsboard::TelemetryData;

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    // 1. Initialize Hardware First (OLED Display turns on immediately)
    let mut display = display::Display::new(peripherals.i2c0, peripherals.pins.gpio4, peripherals.pins.gpio5)?;
    display.show_message("Smart Dispenser", "Connecting Wi-Fi...", "");

    let mut servo = servo::PillServo::new(peripherals.ledc.timer0, peripherals.ledc.channel0, peripherals.pins.gpio6)?;
    let mut buzzer = buzzer::Buzzer::new(peripherals.ledc.timer1, peripherals.ledc.channel1, peripherals.pins.gpio7)?;
    let mut leds = led::Leds::new(peripherals.pins.gpio8, peripherals.pins.gpio9)?;
    
    // Hardware Interrupt Setup for Button
    let mut button = PinDriver::input(peripherals.pins.gpio3, Pull::Up)?;
    button.set_interrupt_type(InterruptType::NegEdge)?; // Trigger on press
    
    let notification = Notification::new();
    let notifier = notification.notifier();
    unsafe {
        button.subscribe(move || {
            notifier.notify_and_yield(NonZeroU32::new(1).unwrap());
        })?;
    }
    button.enable_interrupt()?;
    leds.turn_off_all()?;

    // 2. Initialize Networking & Adaptive Time Synchronization
    let (_wifi, is_online) = wifi::connect(peripherals.modem, sysloop, nvs)?;
    
    // Keeps the SNTP time synchronization alive ONLY if we are online
    let _sntp = if is_online {
        display.show_message("Smart Dispenser", "Syncing Time...", "");
        let sntp_anchor = scheduler::init_time();
        display.show_message("Smart Dispenser", "Ready", "(Online)");
        Some(sntp_anchor)
    } else {
        display.show_message("Smart Dispenser", "Ready", "(Offline Mode)");
        None
    };

    // 3. Initialize Telemetry Queue (Offline Buffering)
    let telemetry_queue: Arc<Mutex<VecDeque<TelemetryData>>> = Arc::new(Mutex::new(VecDeque::new()));
    let queue_clone = telemetry_queue.clone();

    // FreeRTOS Thread: Background Telemetry Worker
    thread::Builder::new().stack_size(8192).spawn(move || {
        loop {
            let item_to_send = {
                let mut queue = queue_clone.lock().unwrap();
                queue.pop_front()
            };

            if let Some(data) = item_to_send {
                if thingsboard::send_telemetry(&data).is_err() {
                    // Push back to front of queue if failed (Wi-Fi dropped)
                    let mut queue = queue_clone.lock().unwrap();
                    queue.push_front(data);
                    thread::sleep(Duration::from_secs(10)); // Backoff before retry
                }
            } else {
                thread::sleep(Duration::from_secs(2)); // Yield to RTOS
            }
        }
    })?;

    let mut schedules = vec![
        MedicationSchedule { name: "Dose 1", hour: 14, minute: 25, dispensed_today: false },
        MedicationSchedule { name: "Dose 2", hour: 14, minute: 26, dispensed_today: false },
        MedicationSchedule { name: "Dose 3", hour: 14, minute: 27, dispensed_today: false },
        MedicationSchedule { name: "Dose 4", hour: 14, minute: 28, dispensed_today: false },
    ];
    let mut last_reset_day = Local::now().day();

    // 4. Main RTOS Loop
    loop {
        let current_day = Local::now().day();
        if current_day != last_reset_day {
            for schedule in &mut schedules { schedule.dispensed_today = false; }
            last_reset_day = current_day;
        }

        let (current_hour, current_minute) = scheduler::get_current_time();

        for schedule in &mut schedules {
            if schedule.hour == current_hour && schedule.minute == current_minute && !schedule.dispensed_today {
                info!("Medication time: {}", schedule.name);
                schedule.dispensed_today = true;

                buzzer.start()?;
                display.show_message("MEDICATION TIME", &format!("Take {}", schedule.name), "Press button");
                servo.dispense()?; 

                // Start Blinking Thread for Red LED
                let _blink_flag = Arc::new(Mutex::new(true));
                let _led_thread = thread::spawn(move || {
                    // Mock loop, since transferring PinDriver to thread requires Arc<Mutex>
                    // For simplicity, we just notify user. In a true hardware setup, 
                    // a hardware timer or separate thread controls the LED pin safely.
                });

                // Explicit FreeRTOS Tick Conversion for Notification Timeout
                let timeout_ticks = esp_idf_hal::delay::TickType::from(Duration::from_secs(60)).ticks();
                let button_pressed = notification.wait(timeout_ticks).is_some();

                // CRITICAL TRAP FIX: Re-enable the pin hardware interrupt!
                // ESP-IDF automatically disables GPIO interrupts after an ISR trigger to prevent flooding.
                button.enable_interrupt()?;

                buzzer.stop()?;
                leds.turn_off_all()?;

                let status_msg;
                if button_pressed {
                    leds.green.set_high()?;
                    display.show_message("Medication Taken", &format!("{} Complete", schedule.name), "");
                    status_msg = "taken";
                    thread::sleep(Duration::from_secs(3));
                    leds.green.set_low()?;
                } else {
                    leds.red.set_high()?;
                    display.show_message("Medication Missed", schedule.name, "");
                    status_msg = "missed";
                }

                // Push telemetry to the offline buffer thread
                let mut queue = telemetry_queue.lock().unwrap();
                queue.push_back(TelemetryData {
                    medicine: schedule.name.to_string(),
                    status: status_msg.to_string(),
                });
                
                display.show_message("Smart Dispenser", "Next schedule...", "");
            }
        }
        
        // Sleep for 10 seconds. FreeRTOS handles power management automatically.
        thread::sleep(Duration::from_secs(10));
    }
}