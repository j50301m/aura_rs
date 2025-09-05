use cfgloader_rs::FromEnv;

#[derive(FromEnv, Debug)]
pub struct Config {
    #[env("PORT", default = "8080")]
    pub port: u16,
}

pub fn load_config() -> Config {
    Config::load(std::path::Path::new(".env")).unwrap()
}
