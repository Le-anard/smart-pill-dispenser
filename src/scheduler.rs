use esp_idf_svc::sntp::{EspSntp, SyncStatus, SntpConf};
use chrono::{Local, Timelike};
use std::thread::sleep;
use std::time::Duration;
use log::info;

#[derive(Clone)]
pub struct MedicationSchedule {
    pub name: &'static str,
    pub hour: u32,
    pub minute: u32,
    pub dispensed_today: bool,
}

pub fn init_time() -> EspSntp<'static> {
    // EAT-3 corresponds to UTC+3 (East Africa Time)
    std::env::set_var("TZ", "EAT-3");
    
    let conf = SntpConf::default();
    let sntp = EspSntp::new(&conf).unwrap();
    
    info!("Waiting for SNTP sync...");
    while sntp.get_sync_status() != SyncStatus::Completed {
        sleep(Duration::from_millis(500));
    }
    info!("Time synchronized!");
    sntp
}

pub fn get_current_time() -> (u32, u32) {
    let now = Local::now();
    (now.hour(), now.minute())
}