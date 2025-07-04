use config::{Config, ConfigError};
use serde::Deserialize;
use std::sync::LazyLock;

use crate::domain::{
    models::{MikrotikConfig, AppConfig, DomainError},
    traits::ConfigService,
};

#[derive(Debug, Deserialize)]
struct ConfigFile {
    app: AppConfigFile,
    mikrotik: MikrotikConfigFile,
}

#[derive(Debug, Deserialize)]
struct AppConfigFile {
    #[serde(default = "default_log_level")]
    log_level: String,
    #[serde(default = "default_bind_address")]
    bind_address: String,
    #[serde(default = "default_bind_port")]
    bind_port: u16,
    #[serde(default = "default_static_files_path")]
    static_files_path: String,
    #[serde(default = "default_session_secret")]
    session_secret: String,
    #[serde(default = "default_ping_interval")]
    ping_interval_seconds: u64,
}

#[derive(Debug, Deserialize)]
struct MikrotikConfigFile {
    protocol: String,
    address: String,
    port: u16,
    username: String,
    password: String,
    #[serde(default = "default_timeout")]
    timeout_seconds: u64,
}

// Default values
fn default_log_level() -> String { "info".to_string() }
fn default_bind_address() -> String { "127.0.0.1".to_string() }
fn default_bind_port() -> u16 { 3217 }
fn default_static_files_path() -> String { "./asset".to_string() }
fn default_session_secret() -> String { "change-me-in-production".to_string() }
fn default_ping_interval() -> u64 { 2 }
fn default_timeout() -> u64 { 10 }

static CONFIG: LazyLock<ConfigFile> = LazyLock::new(|| {
    let builder = Config::builder()
        .add_source(config::File::with_name("config").required(true))
        .add_source(config::Environment::with_prefix("MIKRITING").separator("_"));

    builder
        .build()
        .expect("Failed to build configuration")
        .try_deserialize()
        .expect("Failed to parse configuration file. Make sure config.toml exists and is properly formatted.")
});

pub struct FileConfigService;

impl FileConfigService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FileConfigService {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigService for FileConfigService {
    fn get_mikrotik_config(&self) -> Result<MikrotikConfig, DomainError> {
        let config = &CONFIG.mikrotik;
        
        Ok(MikrotikConfig {
            protocol: config.protocol.clone(),
            address: config.address.clone(),
            port: config.port,
            username: config.username.clone(),
            password: config.password.clone(),
            timeout_seconds: config.timeout_seconds,
        })
    }

    fn get_app_config(&self) -> Result<AppConfig, DomainError> {
        let config = &CONFIG.app;
        
        Ok(AppConfig {
            log_level: config.log_level.clone(),
            bind_address: config.bind_address.clone(),
            bind_port: config.bind_port,
            static_files_path: config.static_files_path.clone(),
            session_secret: config.session_secret.clone(),
            ping_interval_seconds: config.ping_interval_seconds,
        })
    }
}

impl From<ConfigError> for DomainError {
    fn from(err: ConfigError) -> Self {
        DomainError::ConfigurationError(err.to_string())
    }
}
