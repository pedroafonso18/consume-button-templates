use std::collections::HashMap;

use reqwest::{self, Client};
use log::{info, error};

pub async fn send_gupshup_message(apikey: &str, body: &str, conn: (String, String), to: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let mut form: HashMap<&str, &str> = HashMap::new();
    form.insert("channel", "whatsapp");
    form.insert("source", &conn.1);
    form.insert("destination", to);
    form.insert("message", &body);
    form.insert("src.name", &conn.0);

    info!("Sending Gupshup message to {} via source {}", to, &conn.1);
    let response = match client.post("https://api.gupshup.io/wa/api/v1/msg")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("apikey", apikey)
        .header("cache-control", "no-cache")
        .header("Cache-Control", "no-cache")
        .form(&form)
        .send()
        .await {
            Ok(resp) => resp,
            Err(e) => {
                error!("HTTP request to Gupshup failed: {}", e);
                return Err(Box::new(e));
            }
        };

    let status = response.status();
    let resp_text = response.text().await.unwrap_or_else(|_| "<Failed to read response body>".to_string());

    if !status.is_success() {
        error!("Gupshup API returned error status: {}. Body: {}", status, resp_text);
        return Err(format!("Gupshup API error: status {}. Body: {}", status, resp_text).into());
    }

    info!("Gupshup message sent successfully. Status: {}. Body: {}", status, resp_text);
    Ok(())
}
