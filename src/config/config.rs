use dotenv::dotenv;
use std::env;

pub struct EnvVars {
    pub db_url: String,
    pub rabbit_url: String,
    pub api_key_huggy: String,
    pub api_key_gup: String,
    pub api_key_huggy2: String,
    pub db_url_logs: String
}

pub fn load() -> EnvVars {
    if let Err(e) = dotenv() {
        eprintln!("Warning: .env file not loaded: {}",e);
    }
    let db_url = env::var("DB_URL").expect("NO DB_URL FOUND IN THE .ENV FILE!");
    let rabbit_url = env::var("RABBIT_URL").expect("NO RABBIT_URL FOUND IN THE .ENV FILE!");
    let api_key_huggy = env::var("API_KEY_HUGGY").expect("NO API_KEY_HUGGY FOUN IN THE .ENV FILE!");
    let api_key_gup = env::var("API_KEY_GUP").expect("NO API_KEY_GUP FOUND IN THE .ENV FILE!");
    let api_key_huggy2 = env::var("API_KEY_HUGGY2").expect("NO API_KEY_HUGGY2 FOUND IN THE .ENV FILE!");
    let db_url_logs = env::var("DB_URL_LOGS").expect("NO DB_URL_LOGS FOUND IN THE .ENV FILE!");

    EnvVars {
        db_url,
        rabbit_url,
        api_key_huggy,
        api_key_gup,
        api_key_huggy2,
        db_url_logs
    }
}