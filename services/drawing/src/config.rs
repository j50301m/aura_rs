use std::path;

use cfgloader_rs::FromEnv;

#[derive(FromEnv)]
pub struct Config {
    pub db: Db,
    #[env("REDIS_URL", default = "redis://localhost:30079")]
    pub redis_url: String,
    pub rabbitmq: RabbitMq,
}

#[derive(FromEnv)]
pub struct Db {
    #[env("DB_URL", required)]
    pub db_url: String,
    #[env("DB_SCHEMA", default = "public")]
    pub db_schema: String,
    #[env("DB_MAX_CONNECTIONS", default = "30")]
    pub db_max_connections: u32,
    #[env("DB_MIN_CONNECTIONS", default = "5")]
    pub db_min_connections: u32,
    #[env("DB_CONNECT_TIMEOUT", default = "8")]
    pub db_connect_timeout: u64,
    #[env("DB_IDLE_TIMEOUT", default = "8")]
    pub db_idle_timeout: u64,
    #[env("DB_MAX_LIFETIME", default = "8")]
    pub db_max_lifetime: u64,
    #[env("DB_LOGGING", default = "false")]
    pub db_logging: bool,
}

#[derive(FromEnv)]
pub struct RabbitMq {
    #[env("RABBITMQ_HOST", default = "localhost")]
    pub host: String,
    #[env("RABBITMQ_PORT", default = "5672")]
    pub port: u16,
    #[env("RABBITMQ_USERNAME", default = "guest")]
    pub username: String,
    #[env("RABBITMQ_PASSWORD", default = "guest")]
    pub password: String,
    #[env("RABBITMQ_VHOST", default = "/")]
    pub vhost: String,
}

impl Config {
    pub fn new() -> Self {
        let current_path = path::PathBuf::from(".env");
        let workspace_path = path::PathBuf::from("services/drawing/.env");
        Config::load_iter(vec![current_path, workspace_path]).expect("Failed to load configuration")
    }
}
