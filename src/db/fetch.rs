use tokio_postgres::Error;
use log::{info, error};
use deadpool_postgres;
use std::collections::HashMap;

pub async fn fetch_uuid(
    client: &deadpool_postgres::Object, 
) -> Result<HashMap<String, String>, Error> {
    info!("Attempting to fetch UUID from database");

    let mut map: HashMap<String, String> = HashMap::new();

    if let Err(e) = client.execute("SET statement_timeout = '30s'", &[]).await {
        error!("Failed to set statement_timeout: {}", e);
        return Err(e);
    }

    if let Err(e) = client.execute("SET idle_in_transaction_session_timeout = '30s'", &[]).await {
        error!("Failed to set idle_in_transaction_session_timeout: {}", e);
        return Err(e);
    }

    let rows = match client.query(
        "SELECT
            p.uuid,
            c.source
        FROM
            parametros p
        JOIN
            conexoes c ON p.source_name = c.source_name;",
        &[]
    ).await {
        Ok(rows) => rows,
        Err(e) => {
            error!("Failed to execute SELECT query: {}", e);
            return Err(e);
        }
    };

    if rows.is_empty() {
        info!("No rows returned from the query.");
    } else {
        for row in rows {
            let uuid: Result<String, _> = row.try_get("uuid");
            let source: Result<String, _> = row.try_get("source");
            match (uuid, source) {
                (Ok(uuid), Ok(source)) => {
                    map.insert(uuid, source);
                },
                (Err(e), _) | (_, Err(e)) => {
                    error!("Failed to extract uuid/source from row: {}", e);
                }
            }
        }
    }

    Ok(map)
}