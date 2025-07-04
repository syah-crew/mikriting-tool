use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnUser {
    pub id: String,
    pub name: String,
    pub service: Option<String>,
    pub caller_id: Option<String>,
    pub address: String,
    pub uptime: String,
    pub comment: Option<String>,
    pub latency: Option<f64>,
    pub is_active: bool,
}

#[allow(dead_code)]
impl VpnUser {
    pub fn new(
        name: String,
        service: Option<String>,
        caller_id: Option<String>,
        address: String,
        uptime: String,
        comment: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            service,
            caller_id,
            address,
            uptime,
            comment,
            latency: None,
            is_active: true,
        }
    }

    pub fn get_ip_address(&self) -> Option<IpAddr> {
        IpAddr::from_str(&self.address).ok()
    }
    
    pub fn update_latency(&mut self, latency: Option<f64>) {
        self.latency = latency;
    }

    pub fn set_inactive(&mut self) {
        self.is_active = false;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub username: String,
    pub is_authenticated: bool,
}

#[allow(dead_code)]
impl AuthUser {
    pub fn new(username: String) -> Self {
        Self {
            username,
            is_authenticated: true,
        }
    }

    pub fn anonymous() -> Self {
        Self {
            username: "anonymous".to_string(),
            is_authenticated: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyUpdate {
    pub user_name: String,
    pub latency: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub message_type: String,
    pub data: serde_json::Value,
}

impl WebSocketMessage {
    pub fn vpn_users_update(users: Vec<VpnUser>) -> Self {
        Self {
            message_type: "vpn_users".to_string(),
            data: serde_json::to_value(users).unwrap_or_default(),
        }
    }

    pub fn latency_update(user_name: String, latency: Option<f64>) -> Self {
        Self {
            message_type: "latency".to_string(),
            data: serde_json::json!({
                "name": user_name,
                "latency": latency
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MikrotikConfig {
    pub protocol: String,
    pub address: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub timeout_seconds: u64,
}

impl MikrotikConfig {
    pub fn base_url(&self) -> String {
        format!("{}://{}:{}", self.protocol, self.address, self.port)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub log_level: String,
    pub bind_address: String,
    pub bind_port: u16,
    pub static_files_path: String,
    pub session_secret: String,
    pub ping_interval_seconds: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            bind_address: "127.0.0.1".to_string(),
            bind_port: 3217,
            static_files_path: "./asset".to_string(),
            session_secret: "change-me-in-production".to_string(),
            ping_interval_seconds: 2,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Invalid IP address: {0}")]
    InvalidIpAddress(String),
    
    #[error("User not found: {0}")]
    UserNotFound(String),
    
    #[error("Authentication failed")]
    AuthenticationFailed,
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}
