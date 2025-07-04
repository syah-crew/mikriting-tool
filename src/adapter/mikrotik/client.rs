use async_trait::async_trait;
use reqwest::Client;
use std::time::Duration;
use log::{debug, error, info};

use crate::domain::{
    models::{VpnUser, MikrotikConfig, DomainError},
    traits::MikrotikService,
};
use super::types::{MikrotikApiRequest, MikrotikApiMethod, MikrotikPppActiveResponse, MikrotikError};

pub struct MikrotikClient {
    client: Client,
    config: MikrotikConfig,
}

impl MikrotikClient {
    pub fn new(config: MikrotikConfig) -> Result<Self, DomainError> {
        let mut client_builder = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds));

        // Accept invalid certs for HTTPS if needed
        if config.protocol == "https" {
            client_builder = client_builder.danger_accept_invalid_certs(true);
        }

        let client = client_builder
            .build()
            .map_err(|e| DomainError::NetworkError(e.to_string()))?;

        Ok(Self { client, config })
    }

    async fn execute_request(&self, request: MikrotikApiRequest) -> Result<reqwest::Response, MikrotikError> {
        let url = request.build_url(&self.config.base_url());
        debug!("Executing MikroTik request: {} {}", 
               match request.method {
                   MikrotikApiMethod::Get => "GET",
                   MikrotikApiMethod::Post => "POST",
                   MikrotikApiMethod::Put => "PUT",
                   MikrotikApiMethod::Delete => "DELETE",
               }, 
               url);

        let mut req_builder = match request.method {
            MikrotikApiMethod::Get => self.client.get(&url),
            MikrotikApiMethod::Post => self.client.post(&url),
            MikrotikApiMethod::Put => self.client.put(&url),
            MikrotikApiMethod::Delete => self.client.delete(&url),
        };

        // Add authentication
        req_builder = req_builder.basic_auth(&self.config.username, Some(&self.config.password));

        // Add body if present
        if let Some(body) = request.body {
            req_builder = req_builder.json(&body);
        }

        let response = req_builder.send().await?;

        if response.status().is_success() {
            Ok(response)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("MikroTik API error: {} - {}", status, error_text);
            
            if status.as_u16() == 401 {
                Err(MikrotikError::AuthenticationError)
            } else {
                Err(MikrotikError::ApiError(format!("HTTP {}: {}", status, error_text)))
            }
        }
    }

    async fn get_active_connections_raw(&self) -> Result<Vec<MikrotikPppActiveResponse>, MikrotikError> {
        let request = MikrotikApiRequest::get_active_connections();
        let response = self.execute_request(request).await?;
        
        let mikrotik_users: Vec<MikrotikPppActiveResponse> = response.json().await?;
        debug!("Retrieved {} active connections from MikroTik", mikrotik_users.len());
        
        Ok(mikrotik_users)
    }
}

#[async_trait]
impl MikrotikService for MikrotikClient {
    async fn fetch_active_connections(&self) -> Result<Vec<VpnUser>, DomainError> {
        info!("Fetching active connections from MikroTik");
        
        let mikrotik_users = self.get_active_connections_raw().await?;
        
        let vpn_users: Vec<VpnUser> = mikrotik_users
            .into_iter()
            .map(|mikrotik_user| mikrotik_user.into())
            .collect();
        
        info!("Successfully fetched {} VPN users", vpn_users.len());
        Ok(vpn_users)
    }

    async fn disconnect_user(&self, user_name: &str) -> Result<(), DomainError> {
        info!("Disconnecting user: {}", user_name);
        
        // Note: MikroTik REST API requires specific .id to disconnect
        // This is a simplified implementation. In production, you'd need to:
        // 1. First get the connection to find its .id
        // 2. Then use that .id to disconnect
        
        // For now, we'll implement a basic approach
        let request = MikrotikApiRequest::disconnect_user(user_name);
        
        match self.execute_request(request).await {
            Ok(_) => {
                info!("Successfully disconnected user: {}", user_name);
                Ok(())
            }
            Err(MikrotikError::UserNotFound(_)) => {
                error!("User not found for disconnection: {}", user_name);
                Err(DomainError::UserNotFound(user_name.to_string()))
            }
            Err(e) => {
                error!("Failed to disconnect user {}: {}", user_name, e);
                Err(e.into())
            }
        }
    }
}
