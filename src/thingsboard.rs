use esp_idf_svc::http::client::{Configuration, EspHttpConnection};
use serde::{Serialize, Deserialize};
use log::{info, error};

// FIX 1: Change to HTTPS instead of HTTP
const THINGSBOARD_URL: &str = "https://eu.thingsboard.cloud/api/v1/oDX9jEdN4l65ps87Tbpm/telemetry";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TelemetryData {
    pub medicine: String,
    pub status: String,
}

pub fn send_telemetry(data: &TelemetryData) -> anyhow::Result<()> {
    let mut client = EspHttpConnection::new(&Configuration {
        use_global_ca_store: true,
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
        ..Default::default()
    })?;

    let payload = serde_json::to_string(data)?;
    
    let content_length_str = payload.len().to_string(); 
    
    let headers = [
        ("Content-Type", "application/json"),
        ("Content-Length", content_length_str.as_str()),
    ];

    client.initiate_request(esp_idf_svc::http::Method::Post, THINGSBOARD_URL, &headers)?;
    
    // Write the JSON payload to the connection
    client.write_all(payload.as_bytes())?;
    
    // FIX 2: Finalize the request and wait for the server to reply!
    client.initiate_response()?;
    
    let status = client.status();
    if status == 200 {
        info!("Telemetry sent successfully: {:?}", data);
        Ok(())
    } else {
        error!("Telemetry failed with status: {}", status);
        Err(anyhow::anyhow!("HTTP Status {}", status))
    }
}