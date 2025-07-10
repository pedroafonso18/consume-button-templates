use lapin::{
    options::{BasicConsumeOptions, BasicQosOptions, QueueDeclareOptions},
    types::FieldTable,
    ConnectionProperties, Consumer, Connection
};
use log::{info, error};
use std::time::Duration;
use tokio::time::sleep;

pub async fn connect_rabbitmq(rabbit_url: &str) -> Result<Connection, lapin::Error> {
    let options = ConnectionProperties::default()
        .with_connection_name("button_consumer".into());
    Connection::connect(rabbit_url, options).await
}

pub async fn setup_consumer(connection: &Connection) -> Result<Consumer, lapin::Error> {
    let channel = connection.create_channel().await?;
    
    channel.basic_qos(1, BasicQosOptions::default()).await?;
    
    let queue_options = QueueDeclareOptions {
        passive: true,
        durable: true,
        ..QueueDeclareOptions::default()
    };
    
    channel.queue_declare("button_templates", queue_options, FieldTable::default()).await?;
    
    let consumer = channel
        .basic_consume(
            "button_templates",
            "button-consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;
    
    Ok(consumer)
}

pub async fn create_rabbitmq_consumer(rabbit_url: &str) -> Option<(Consumer, Connection)> {
    loop {
        match connect_rabbitmq(rabbit_url).await {
            Ok(connection) => {
                info!("RabbitMQ connection established");
                match setup_consumer(&connection).await {
                    Ok(consumer) => {
                        info!("Consumer set up successfully");
                        return Some((consumer, connection));
                    }
                    Err(e) => {
                        error!("Failed to set up consumer: {}", e);
                        sleep(Duration::from_secs(5)).await;
                    }
                }
            }
            Err(e) => {
                error!("Failed to connect to RabbitMQ: {}", e);
                sleep(Duration::from_secs(5)).await;
            }
        }
    }
}
