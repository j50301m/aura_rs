use std::path;

use cfgloader_rs::FromEnv;

#[derive(FromEnv)]
pub struct Config {
    pub db: Db,
}

#[derive(FromEnv)]
pub struct Db {
    #[env("DB_URL")]
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

impl Config {
    pub fn new() -> Self {
        Config::load(&path::PathBuf::from(".env")).expect("Failed to load config from env")
    }
}
