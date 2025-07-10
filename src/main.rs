mod config;
mod rabbit;
mod db;
use env_logger::{Builder, Env};
use rabbit::{connect as rmq_connect};
use log::{error, info, warn};



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

    loop {
        match run_
    }

}