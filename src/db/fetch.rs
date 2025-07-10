use tokio_postgres::Error;
use log::{info, error};
use deadpool_postgres;

pub async fn fetch_uuid(
    client: &deadpool_postgres::Object,
    source: &str
) -> Result<Option<String>, Error> {
    info!("Attempting to fetch UUID from database for source: {}", source);

    if let Err(e) = client.execute("SET statement_timeout = '30s'", &[]).await {
        error!("Failed to set statement_timeout: {}", e);
        return Err(e);
    }

    if let Err(e) = client.execute("SET idle_in_transaction_session_timeout = '30s'", &[]).await {
        error!("Failed to set idle_in_transaction_session_timeout: {}", e);
        return Err(e);
    }

    let row = match client.query_opt(
        "SELECT p.uuid FROM parametros p JOIN conexoes c ON p.source_name = c.source_name WHERE c.source = $1",
        &[&source]
    ).await {
        Ok(row) => row,
        Err(e) => {
            error!("Failed to execute SELECT query: {}", e);
            return Err(e);
        }
    };

    match row {
        Some(row) => {
            match row.try_get::<_, String>("uuid") {
                Ok(uuid) => Ok(Some(uuid)),
                Err(e) => {
                    error!("Failed to extract uuid from row: {}", e);
                    Ok(None)
                }
            }
        },
        None => {
            info!("No uuid found for source: {}", source);
            Ok(None)
        }
    }
}

pub async fn fetch_conn(
    client: &deadpool_postgres::Object,
    source: &str
) -> Result<Option<String>, Error> {
    info!("Attempting to fetch conn from database for source: {}", source);

    if let Err(e) = client.execute("SET statement_timeout = '30s'", &[]).await {
        error!("Failed to set statement_timeout: {}", e);
        return Err(e);
    }

    if let Err(e) = client.execute("SET idle_in_transaction_session_timeout = '30s'", &[]).await {
        error!("Failed to set idle_in_transaction_session_timeout: {}", e);
        return Err(e);
    }

    let row = match client.query_opt(
        "SELECT source_name FROM parametros WHERE source_name IN (SELECT source_name FROM conexoes WHERE source = $1)",
        &[&source]
    ).await {
        Ok(row) => row,
        Err(e) => {
            error!("Failed to execute SELECT query: {}", e);
            return Err(e);
        }
    };

    match row {
        Some(row) => {
            match row.try_get::<_, String>("source_name") {
                Ok(source_name) => Ok(Some(source_name)),
                Err(e) => {
                    error!("Failed to extract source_name from row: {}", e);
                    Ok(None)
                }
            }
        },
        None => {
            info!("No source_name found for source: {}", source);
            Ok(None)
        }
    }
}