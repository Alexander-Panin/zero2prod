#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

use config::{Config, ConfigError, File, FileFormat};

pub fn get_configuration() -> Result<Settings, ConfigError> {
    Config::builder()
        .add_source(File::new("configuration.yaml", FileFormat::Yaml))
        .build()
        .unwrap()
        .try_deserialize()
}
