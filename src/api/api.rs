use std::collections::HashMap;

use reqwest::{self, Client};
use log::{info, error};

async fn send_gupshup_message(apikey: &str, body: &str, conn: (String, String), to: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let mut form: HashMap<&str, &str> = HashMap::new();
    let message = format!("{{\"type\":\"text\",\"text\":\"{}\"}}", body);
    form.insert("channel", "whatsapp");
    form.insert("source", &conn.1);
    form.insert("destination", to);
    form.insert("message", &message);
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

pub async fn create_contact(apikey: &str, uuid: &str, conn: (String, String), to: &str, apikey_gup: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    
    info!("Creating contact in Huggy for phone: {}", to);
    
    let contact_payload = serde_json::json!({
        "name": format!("{} API", to),
        "phone": to,
        "email": format!("sem{}@email.com", to)
    });
    
    let create_response = match client.post("https://api.huggy.app/v2/contacts")
        .header("Authorization", format!("Bearer {}", apikey))
        .header("cookie", "app_powerzap=nks19drvheb20cffjlh7fqatqb;")
        .header("Content-Type", "application/json")
        .body(contact_payload.to_string())
        .send()
        .await {
            Ok(resp) => resp,
            Err(e) => {
                error!("Failed to create contact in Huggy: {}", e);
                return Err(Box::new(e));
            }
        };
    
    let status = create_response.status();
    let resp_text = create_response.text().await.unwrap_or_else(|_| "<Failed to read response body>".to_string());
    
    info!("Contact creation response: status={}, body={}", status, resp_text);
    
    let contact_id = if status.is_success() {
        match serde_json::from_str::<serde_json::Value>(&resp_text) {
            Ok(json) => {
                if let Some(id) = json.get("id").and_then(|v| v.as_str()) {
                    Some(id.to_string())
                } else if let Some(arr) = json.as_array() {
                    arr.first().and_then(|v| v.get("id")).and_then(|v| v.as_str()).map(|s| s.to_string())
                } else {
                    None
                }
            },
            Err(e) => {
                error!("Failed to parse contact creation response: {}", e);
                None
            }
        }
    } else {
        if resp_text.contains("já existe um contato") {
            info!("Contact already exists, will search for existing contact");
            let search_response = match client.get(&format!("https://api.huggy.app/v2/contacts?phone={}", to))
                .header("Authorization", format!("Bearer {}", apikey))
                .header("cookie", "app_powerzap=nks19drvheb20cffjlh7fqatqb;")
                .send()
                .await {
                    Ok(resp) => resp,
                    Err(e) => {
                        error!("Failed to search for existing contact: {}", e);
                        return Err(Box::new(e));
                    }
                };
            
            if search_response.status().is_success() {
                let search_text = search_response.text().await.unwrap_or_else(|_| "[]".to_string());
                match serde_json::from_str::<serde_json::Value>(&search_text) {
                    Ok(json) => {
                        if let Some(arr) = json.as_array() {
                            arr.first().and_then(|v| v.get("id")).and_then(|v| v.as_str()).map(|s| s.to_string())
                        } else {
                            None
                        }
                    },
                    Err(e) => {
                        error!("Failed to parse contact search response: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            error!("Contact creation failed with status {}: {}", status, resp_text);
            return Err(format!("Contact creation failed: status {}. Body: {}", status, resp_text).into());
        }
    };
    
    let contact_id = match contact_id {
        Some(id) => id,
        None => {
            error!("Could not extract contact ID from response");
            return Err("Could not extract contact ID from response".into());
        }
    };
    
    info!("Using contact ID: {}", contact_id);
    
    let timeline_response = match client.get(&format!("https://api.huggy.app/v3/contacts/{}/timeline", contact_id))
        .header("Authorization", format!("Bearer {}", apikey))
        .header("cookie", "app_powerzap=nks19drvheb20cffjlh7fqatqb;")
        .send()
        .await {
            Ok(resp) => resp,
            Err(e) => {
                error!("Failed to fetch contact timeline: {}", e);
                return Err(Box::new(e));
            }
        };
    
    if !timeline_response.status().is_success() {
        error!("Failed to fetch timeline: status={}", timeline_response.status());
        return Err(format!("Failed to fetch timeline: status {}", timeline_response.status()).into());
    }
    
    let timeline_text = timeline_response.text().await.unwrap_or_else(|_| "[]".to_string());
    let timeline: Vec<serde_json::Value> = match serde_json::from_str(&timeline_text) {
        Ok(timeline) => timeline,
        Err(e) => {
            error!("Failed to parse timeline response: {}", e);
            return Err(Box::new(e));
        }
    };
    
    let current_chat = timeline.iter()
        .filter(|item| {
            item.get("deleted").and_then(|v| v.as_bool()) == Some(false) &&
            item.get("title").and_then(|v| v.as_str()) == Some("Started chat")
        })
        .max_by_key(|item| {
            if let Some(time) = item.get("time").and_then(|v| v.as_str()) {
                // Parse time string to timestamp
                chrono::NaiveDateTime::parse_from_str(time, "%Y/%m/%d %H:%M:%S")
                    .unwrap_or_else(|_| chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap())
                    .timestamp()
            } else {
                item.get("chatID").and_then(|v| v.as_u64()).unwrap_or(0) as i64
            }
        });
    
    if let Some(chat) = current_chat {
        if let Some(chat_id) = chat.get("chatID").and_then(|v| v.as_str()) {
            info!("Found existing chat ID: {}", chat_id);
            
            let chat_response = match client.get(&format!("https://api.huggy.app/v3/chats/{}", chat_id))
                .header("Authorization", format!("Bearer {}", apikey))
                .header("cookie", "app_powerzap=nks19drvheb20cffjlh7fqatqb;")
                .send()
                .await {
                    Ok(resp) => resp,
                    Err(e) => {
                        error!("Failed to fetch chat details: {}", e);
                        return Err(Box::new(e));
                    }
                };
            
            if chat_response.status().is_success() {
                let chat_data_text = chat_response.text().await.unwrap_or_else(|_| "{}".to_string());
                match serde_json::from_str::<serde_json::Value>(&chat_data_text) {
                    Ok(chat_data) => {
                        let situation = chat_data.get("situation").and_then(|v| v.as_str());
                        let enabled_session = chat_data.get("enabledSession").and_then(|v| v.as_bool());
                        
                        if situation == Some("finishing") || enabled_session == Some(false) {
                            info!("Chat is closed, will create new chat");
                        } else {
                            info!("Chat is still open, no action needed");
                            return Ok(());
                        }
                    },
                    Err(e) => {
                        error!("Failed to parse chat response: {}", e);
                        return Err(Box::new(e));
                    }
                }
            }
        }
    }
    
    info!("Creating new chat for contact");
    let chat_payload = serde_json::json!({
        "uuid": uuid,
        "flowId": "452128",
        "whenInChat": true,
        "whenWaitForChat": true,
        "whenInAuto": true
    });
    
    let flow_response = match client.put(&format!("https://api.huggy.app/v3/contacts/{}/ExecFlow", contact_id))
        .header("Authorization", format!("Bearer {}", apikey))
        .header("cookie", "app_powerzap=nks19drvheb20cffjlh7fqatqb;")
        .header("Content-Type", "application/json")
        .body(chat_payload.to_string())
        .send()
        .await {
            Ok(resp) => resp,
            Err(e) => {
                error!("Failed to execute flow: {}", e);
                return Err(Box::new(e));
            }
        };
    
    let flow_status = flow_response.status();
    let flow_resp_text = flow_response.text().await.unwrap_or_else(|_| "<Failed to read response body>".to_string());
    
    if !flow_status.is_success() {
        error!("Flow execution failed: status={}, body={}", flow_status, flow_resp_text);
        return Err(format!("Flow execution failed: status {}. Body: {}", flow_status, flow_resp_text).into());
    }
    
    info!("Contact creation and chat setup completed successfully");
    
    let message = "Perfeito! Agora, você saberia me informar se ainda tem acesso ao aplicativo do FGTS?\n\nDigite: 1 para Sim!\nDigite: 2 para Não!";
    if let Err(e) = send_gupshup_message(apikey_gup, message, conn, to).await {
        error!("Failed to send Gupshup message after contact creation: {}", e);
    } else {
        info!("Gupshup message sent successfully after contact creation");
    }
    
    Ok(())
}