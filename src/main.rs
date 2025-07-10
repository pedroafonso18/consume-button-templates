mod config;
mod rabbit;
mod process;
mod db;
mod api;
use env_logger::{Builder, Env};
use log::{error, info, warn};
use std::sync::Arc;
use rabbit::{connect as rmq_connect};
use tokio::select;
use tokio::signal;
use futures::pin_mut;
use futures::StreamExt;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = Env::default().filter_or("RUST_LOG", "debug");

    Builder::from_env(env)
        .format_timestamp_secs()
        .format_module_path(true)
        .init();

    println!("Starting application - Check logs below:");

    info!("Starting button consumer application");
    info!("Log level is set - if you see this message, logging is working!");

    let env_vars = config::config::load();

    let db_pool = db::connect::create_pool(&env_vars.db_url).await?;
    let db_logs_pool = db::connect::create_pool(&env_vars.db_url_logs).await?;

    loop {
        let db_client = db_pool.get().await?;
        let db_logs = db_logs_pool.get().await?;
        let db_client = Arc::new(db_client);
        let db_logs = Arc::new(db_logs);

        match run_consumer(&env_vars.rabbit_url, &db_client, &env_vars.api_key_gup, &env_vars.api_key_huggy, &env_vars.api_key_huggy2, &db_logs).await {
            Ok(_) => {
                info!("Application shutdown requested");
                break;
            }
            Err(e) => {
                error!("Error in consumer loop: {}", e);
                println!("ERROR: Consumer loop failed: {}", e);
                info!("Reconnecting in 5 seconds...");
                sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }

    Ok(())
}

async fn run_consumer(
    rabbit_url: &str,
    db_client: &Arc<deadpool_postgres::Object>,
    api_key_gup: &str,
    api_key_hug: &str,
    api_key_hug2: &str,
    db_logs: &Arc<deadpool_postgres::Object>
) -> Result<(), Box<dyn std::error::Error>> {
    let (mut consumer, _) = match rmq_connect::create_rabbitmq_consumer(rabbit_url).await {
        Some((consumer, connection)) => (consumer, connection),
        None => return Err("Failed to create RabbitMQ consumer".into()),
    };

    info!("Consumer ready, waiting for webhooks...");
    info!("Press Ctrl+C to exit");

    loop {
        let ctrl_c_future = signal::ctrl_c();
        pin_mut!(ctrl_c_future);

        select! {
            delivery_result = consumer.next() => {
                match delivery_result {
                    Some(Ok(delivery)) => {
                        let data = delivery.data.clone();
                        let db = Arc::clone(db_client);
                        let api_key_hug = api_key_hug.to_string();
                        let api_key_gup = api_key_gup.to_string();
                        let api_key_hug2 = api_key_hug2.to_string();
                        let db_logs = Arc::clone(db_logs);

                        let handle = tokio::spawn(async move {
                            info!("Starting webhook processing in spawned task");
                            match process::process::process_webhook(&data, &db, &api_key_hug, &api_key_gup, &api_key_hug2, &db_logs).await {
                                Ok(_) => {
                                    info!("Successfully processed webhook in spawned task");
                                },
                                Err(e) => {
                                    error!("Error processing webhook in spawned task: {}", e);
                                }
                            }
                        });

                        if let Err(e) = handle.await {
                            error!("Spawned task failed: {}", e);
                        }

                        if let Err(e) = delivery.ack(lapin::options::BasicAckOptions::default()).await {
                            error!("Failed to acknowledge message: {}", e);
                        }
                    },
                    Some(Err(e)) => {
                        error!("Error receiving message: {}", e);
                        return Err(Box::new(e));
                    },
                    None => {
                        warn!("Consumer channel closed");
                        return Err("Consumer channel closed unexpectedly".into());
                    }
                }
            },

            _ = ctrl_c_future => {
                info!("Received shutdown signal");
                break;
            }
        }
    }

    Ok(())
}