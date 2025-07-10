use serde::{Deserialize, Serialize};
use log::{info, error};
use deadpool_postgres::Object;

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppWebhook {
    pub entry: Vec<Entry>,
    pub gs_app_id: String,
    pub object: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Entry {
    pub changes: Vec<Change>,
    pub id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Change {
    pub field: String,
    pub value: Value,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Value {
    pub contacts: Vec<Contact>,
    pub messages: Vec<Message>,
    pub messaging_product: String,
    pub metadata: Metadata,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Contact {
    pub profile: Profile,
    pub wa_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Profile {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
    pub button: Option<Button>,
    pub context: Option<Context>,
    pub from: String,
    pub id: String,
    pub timestamp: String,
    pub r#type: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Button {
    pub payload: String,
    pub text: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Context {
    pub from: String,
    pub gs_id: String,
    pub id: String,
    pub meta_msg_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Metadata {
    pub display_phone_number: String,
    pub phone_number_id: String,
}

pub fn parse_webhook_data(data: &[u8]) -> Result<Option<String>, Box<dyn std::error::Error>> {
    info!("Parsing webhook data");
    let json_data = String::from_utf8_lossy(data);
    
    let webhook: WhatsAppWebhook = match serde_json::from_str(&json_data) {
        Ok(data) => data,
        Err(e) => {
            error!("Failed to deserialize webhook JSON: {}", e);
            return Err(Box::new(e));
        }
    };
    
    info!("Webhook deserialized successfully, processing {} entries", webhook.entry.len());
    
    for entry in webhook.entry {
        for change in entry.changes {
            if change.field == "messages" {
                info!("Processing messages change");
                
                for message in change.value.messages {
                    if let Some(context) = message.context {
                        info!("Found message with context, from: {}", context.from);
                        return Ok(Some(context.from));
                    } else {
                        info!("Message has no context, skipping");
                    }
                }
            }
        }
    }
    
    info!("No context.from found in webhook data");
    Ok(None)
}

pub async fn process_webhook(
    data: &[u8],
    db_client: &Object,
    api_key_hug: &str,
    api_key_gup: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Processing webhook...");
    let source = match parse_webhook_data(data)? {
        Some(s) => s,
        None => {
            error!("No source (context.from) found in webhook");
            return Err("No source (context.from) found in webhook".into());
        }
    };
    info!("Extracted source: {}", source);

    let uuid = match crate::db::fetch::fetch_uuid(db_client, &source).await? {
        Some(u) => u,
        None => {
            error!("No uuid found for source: {}", source);
            return Err("No uuid found for source".into());
        }
    };
    info!("Fetched uuid: {}", uuid);

    let conn = match crate::db::fetch::fetch_conn(db_client, &source).await? {
        Some(c) => c,
        None => {
            error!("No connection found for source: {}", source);
            return Err("No connection found for source".into());
        }
    };
    info!("Fetched connection: {}", conn);

    let conn_tuple = (conn.clone(), source.clone());
    crate::api::api::create_contact(api_key_hug, &uuid, conn_tuple, &source, api_key_gup).await?;
    info!("Contact creation process completed successfully");
    Ok(())
}
