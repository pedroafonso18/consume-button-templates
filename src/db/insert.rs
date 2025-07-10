use tokio_postgres::Error;
use log::{info, error};
use deadpool_postgres;

pub async fn insert_log(
    client: &deadpool_postgres::Object,
    num: &str,
    msg: &str,
    resp_cliente: &str
) -> Result<(), Error> {
    info!("Attempting to insert log into the database:");

    if let Err(e) = client.execute("SET statement_timeout = '30s'", &[]).await {
        error!("Failed to set statement_timeout: {}", e);
        return Err(e);
    }

    if let Err(e) = client.execute("SET idle_in_transaction_session_timeout = '30s'", &[]).await {
        error!("Failed to set idle_in_transaction_session_timeout: {}", e);
        return Err(e);
    }

    match client.execute(
        "INSERT INTO \"button-answers\" (num, mensagem, resposta_cliente) VALUES ($1, $2, $3)",
        &[&num, &msg, &resp_cliente]
    ).await {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Failed to execute INSERT query: {}", e);
            return Err(e);
        }
    }
}
