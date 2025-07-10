use tokio_postgres::Config;
use deadpool_postgres::{Pool, Manager};
use std::time::Duration;


pub async fn create_pool(url: &str) -> Result<Pool, Box<dyn std::error::Error>> {    
    let mut cfg: Config = url.parse()?;
    
    cfg.keepalives_idle(Duration::from_secs(30));
    cfg.keepalives_interval(Duration::from_secs(10));
    cfg.keepalives_retries(5);
    
    let mgr = Manager::new(cfg, tokio_postgres::NoTls);
    let pool = Pool::builder(mgr)
        .max_size(16)
        .build()?;
    
    let _ = pool.get().await?;
    
    Ok(pool)
}