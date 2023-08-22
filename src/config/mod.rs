use config::{Config, ConfigError, File};
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;

use crate::db::{create_pg_connection_pool, PG_POOL};

lazy_static! {
    pub static ref CONFIG: OnceCell<Settings> = OnceCell::new();
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
}

#[derive(Debug, Deserialize)]
pub struct DiscordConfig {
    pub auth_token: String,
}

#[derive(Debug, Deserialize)]
pub struct RedisConfig {
    pub connection_string: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub discord: DiscordConfig,
    pub redis: RedisConfig,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();

        s.set_default("logging.level", "warn").unwrap();

        // Start off by merging in the "default" configuration file
        s.merge(File::with_name("config/default.yml"))?;

        // Add in a local configuration file
        // This file shouldn't be checked in to git
        s.merge(File::with_name("config/config.yml").required(false))?;

        match s.get_str("discord.auth_token") {
            Ok(cs) => {
                let csb = cs.clone().into_bytes();
                if csb.len() != 59 {
                    return Err(ConfigError::Message(format!(
                        "Auth token must be exactly 59 bytes, {} provided",
                        csb.len()
                    )));
                };
            }
            Err(_) => {
                return Err(ConfigError::Message("Auth token not set".to_owned()));
            }
        };

        s.try_into()
    }
}

pub fn load_config() -> Result<Settings, ConfigError> {
    // Parse config here
    match Settings::new() {
        Ok(settings) => {
            // Now that we're done, let's access our configuration
            println!("Logging level: {:?}", &settings.logging.level);
            println!("Connecting to Database...");
            // Create DB Connection Pool
            PG_POOL
                .set(create_pg_connection_pool(&settings.database.url))
                .map_err(|_| ())
                .expect("Failed to initalise DB Connection pool");

            Ok(settings)
        }
        Err(e) => Err(e),
    }
}
