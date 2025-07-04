use crate::domain::models::VpnUser;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MikrotikPppActiveResponse {
    #[serde(rename = ".id")]
    pub id: String,
    pub name: String,
    pub service: Option<String>,
    #[serde(rename = "caller-id")]
    pub caller_id: Option<String>,
    pub address: String,
    pub uptime: String,
    pub comment: Option<String>,
}

impl From<MikrotikPppActiveResponse> for VpnUser {
    fn from(mikrotik_user: MikrotikPppActiveResponse) -> Self {
        VpnUser::new(
            mikrotik_user.name,
            mikrotik_user.service,
            mikrotik_user.caller_id,
            mikrotik_user.address,
            mikrotik_user.uptime,
            mikrotik_user.comment,
        )
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum MikrotikApiPath {
    PppActive,
    PppActiveById(String),
    PppSecrets,
    PppSecretById(String),
    InterfaceMonitor,
}

impl MikrotikApiPath {
    pub fn to_path(&self) -> String {
        match self {
            MikrotikApiPath::PppActive => "/rest/ppp/active".to_string(),
            MikrotikApiPath::PppActiveById(id) => format!("/rest/ppp/active/{}", id),
            MikrotikApiPath::PppSecrets => "/rest/ppp/secret".to_string(),
            MikrotikApiPath::PppSecretById(id) => format!("/rest/ppp/secret/{}", id),
            MikrotikApiPath::InterfaceMonitor => "/rest/interface/monitor-traffic".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MikrotikApiRequest {
    pub path: MikrotikApiPath,
    pub method: MikrotikApiMethod,
    pub query_params: Vec<(String, String)>,
    pub body: Option<serde_json::Value>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum MikrotikApiMethod {
    Get,
    Post,
    Put,
    Delete,
}

impl MikrotikApiRequest {
    pub fn new(path: MikrotikApiPath, method: MikrotikApiMethod) -> Self {
        Self {
            path,
            method,
            query_params: Vec::new(),
            body: None,
        }
    }

    pub fn with_query_param(mut self, key: String, value: String) -> Self {
        self.query_params.push((key, value));
        self
    }

    #[allow(dead_code)]
    pub fn with_body(mut self, body: serde_json::Value) -> Self {
        self.body = Some(body);
        self
    }

    pub fn build_url(&self, base_url: &str) -> String {
        let mut url = format!("{}{}", base_url, self.path.to_path());

        if !self.query_params.is_empty() {
            url.push('?');
            let params: Vec<String> = self.query_params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            url.push_str(&params.join("&"));
        }

        url
    }
}

// Predefined requests for common operations
impl MikrotikApiRequest {
    pub fn get_active_connections() -> Self {
        Self::new(MikrotikApiPath::PppActive, MikrotikApiMethod::Get)
            .with_query_param(".proplist".to_string(), "name,service,caller-id,address,uptime,comment".to_string())
    }

    pub fn disconnect_user(user_name: &str) -> Self {
        // In MikroTik, disconnecting requires finding the connection ID first
        // This is a simplified approach - in real implementation, you'd need to:
        // 1. Get the connection by name
        // 2. Use the .id to disconnect
        Self::new(MikrotikApiPath::PppActive, MikrotikApiMethod::Delete)
            .with_query_param("name".to_string(), user_name.to_string())
    }

    #[allow(dead_code)]
    pub fn get_user_details(user_name: &str) -> Self {
        Self::new(MikrotikApiPath::PppActive, MikrotikApiMethod::Get)
            .with_query_param("name".to_string(), user_name.to_string())
    }
}

#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum MikrotikError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Authentication failed")]
    AuthenticationError,

    #[error("API response error: {0}")]
    ApiError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Connection timeout")]
    Timeout,
}

impl From<MikrotikError> for crate::domain::models::DomainError {
    fn from(err: MikrotikError) -> Self {
        match err {
            MikrotikError::HttpError(e) => crate::domain::models::DomainError::NetworkError(e.to_string()),
            MikrotikError::AuthenticationError => crate::domain::models::DomainError::AuthenticationFailed,
            MikrotikError::ApiError(msg) => crate::domain::models::DomainError::NetworkError(msg),
            MikrotikError::SerializationError(e) => crate::domain::models::DomainError::SerializationError(e.to_string()),
            MikrotikError::UserNotFound(name) => crate::domain::models::DomainError::UserNotFound(name),
            MikrotikError::Timeout => crate::domain::models::DomainError::NetworkError("Request timeout".to_string()),
        }
    }
}
