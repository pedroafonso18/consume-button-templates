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

pub fn parse_webhook_data(data: &[u8]) -> Result<Option<(String, String, String)>, Box<dyn std::error::Error>> {
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
                        let context_from = context.from;
                        let message_from = message.from;
                        
                        if let Some(button) = message.button {
                            let button_text = button.text;
                            info!("Found message with context and button, context.from: {}, message.from: {}, button.text: {}", context_from, message_from, button_text);
                            return Ok(Some((context_from, message_from, button_text)));
                        } else {
                            info!("Message has context but no button, skipping");
                        }
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
    api_key_hug2: &str,
    db_client_logs: &Object
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Processing webhook...");
    let (source, whatsapp_number, button_text, ) = match parse_webhook_data(data)? {
        Some((context_from, message_from, button_text)) => (context_from, message_from, button_text),
        None => {
            error!("No source (context.from) found in webhook");
            return Err("No source (context.from) found in webhook".into());
        }
    };
    info!("Extracted source: {}, WhatsApp number: {}, button text: {}", source, whatsapp_number, button_text);
    let button_text_lwr = button_text.to_lowercase();
    info!("Button text validation passed, continuing with processing");

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

    let conn_tuple = (conn, source);
    if button_text_lwr.contains("sim") || button_text_lwr.contains("chamar") {
        crate::api::api::send_gupshup_message(api_key_gup, "Vamos lá! Antes de realizar a consulta, é importante saber: o empréstimo do Bolsa Família pode chegar até R$650, caso o seu benefício esteja liberado.\n\nAtualmente, você recebe o Bolsa Família pelo aplicativo Caixa Tem?\n\nDigite:\n1️⃣ Sim\n2️⃣ Não", conn_tuple, &whatsapp_number).await?;
        match crate::db::insert::insert_log(&db_client_logs, &whatsapp_number, "Perfeito! Agora, você saberia me informar se ainda tem acesso ao aplicativo do FGTS?\n\nDigite: 1 para Sim!\nDigite: 2 para Não!\n", &button_text, "FGTS").await {
            Ok(_) => {
                info!("Contact creation process completed successfully");
                Ok(())        
            },
            Err(e) => {
                error!("Error when inserting log: {}",e);
                Ok(())
            }
        }
    } else {
        
        crate::api::api::send_gupshup_message(api_key_gup, "Perfeito! 😊\nAgora, você saberia me informar se ainda tem acesso ao aplicativo do FGTS?\n\nDigite:\n1️⃣ Sim, tenho acesso!\n2️⃣ Não tenho!", conn_tuple, &whatsapp_number).await?;
        match crate::db::insert::insert_log(&db_client_logs, &whatsapp_number, "Perfeito! Agora, você saberia me informar se ainda tem acesso ao aplicativo do FGTS?\n\nDigite: 1 para Sim!\nDigite: 2 para Não!\n", &button_text, "BOLSA").await {
            Ok(_) => {
                info!("Contact creation process completed successfully");
                Ok(())        
            },
            Err(e) => {
                error!("Error when inserting log: {}",e);
                Ok(())
            }
        }
    } 
}
