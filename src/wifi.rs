use esp_idf_svc::wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_hal::modem::Modem;
use log::{info, warn};

const SSID: &str = "Redmi 15";
const PASSWORD: &str = "12345678";

/// Connects to Wi-Fi. Returns the Wi-Fi driver instance along with a boolean 
/// indicating if the network connection was successfully established with an IP address.
pub fn connect<'a>(
    modem: Modem<'a>, 
    sysloop: EspSystemEventLoop,
    nvs: EspDefaultNvsPartition,
) -> anyhow::Result<(BlockingWifi<EspWifi<'a>>, bool)> {
    
    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(modem, sysloop.clone(), Some(nvs))?,
        sysloop,
    )?;

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: SSID.try_into().unwrap(),
        password: PASSWORD.try_into().unwrap(),
        auth_method: AuthMethod::WPA2Personal,
        ..Default::default()
    }))?;

    wifi.start()?;
    
    info!("Attempting to connect to Wi-Fi SSID: {}...", SSID);
    
    let is_online = match wifi.connect() {
        Ok(_) => {
            info!("Connected to access point. Requesting IP address...");
            match wifi.wait_netif_up() {
                Ok(_) => {
                    info!("Wi-Fi Connection fully established!");
                    true
                }
                Err(e) => {
                    warn!("Connected to router, but DHCP failed to assign an IP: {:?}", e);
                    false
                }
            }
        }
        Err(e) => {
            warn!("Wi-Fi network connection failed: {:?}", e);
            false
        }
    };

    if !is_online {
        warn!("System booting up in OFFLINE mode. Network features will be bypassed.");
    }

    Ok((wifi, is_online))
}