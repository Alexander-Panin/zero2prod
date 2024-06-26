use crate::domain::SubscriberEmail;
use config::{Config, ConfigError, File, FileFormat};
use std::convert::{TryFrom, TryInto};

#[derive(serde::Deserialize, Debug)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub email_client: EmailClientSettings,
}

#[derive(serde::Deserialize, Debug)]
pub struct EmailClientSettings {
    pub base_url: String,
    pub sender_email: String,
}

impl EmailClientSettings {
    pub fn sender(&self) -> Result<SubscriberEmail, String> {
        SubscriberEmail::parse(self.sender_email.clone())
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct DatabaseSettings {
    pub filename: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        self.filename.to_string()
    }
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");
    let env_config = format!("configuration/{}.yaml", environment.as_str());
    Config::builder()
        .add_source(File::new("configuration/base.yaml", FileFormat::Yaml))
        .add_source(File::new(&env_config, FileFormat::Yaml))
        .build()
        .unwrap()
        .try_deserialize()
}

pub enum Environment {
    Local,
    Production,
    Test,
}

impl Environment {
    pub fn as_str(&self) -> &str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
            Environment::Test => "test",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            "test" => Ok(Self::Test),
            other => Err(format!(
                "{} is not a supported environment. Use either `local`, `production` or `test`.",
                other
            )),
        }
    }
}
