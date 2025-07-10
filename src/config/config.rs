use dotenv::dotenv;
use std::env;

pub struct EnvVars {
    pub db_url: String,
    pub rabbit_url: String,
}

pub fn load() -> EnvVars {
    if let Err(e) = dotenv() {
        eprintln!("Warning: .env file not loaded: {}",e);
    }
    let db_url = env::var("DB_URL").expect("NO DB_URL FOUND IN THE .ENV FILE!");
    let rabbit_url = env::var("RABBIT_URL").expect("NO RABBIT_URL FOUND IN THE .ENV FILE!");

    EnvVars {
        db_url,
        rabbit_url,
    }
}