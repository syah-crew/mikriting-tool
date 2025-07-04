use async_trait::async_trait;
use htpasswd_verify::Htpasswd;
use std::fs;
use tokio::task;
use log::{debug, error};

use crate::domain::{
    models::{AuthUser, DomainError},
    traits::AuthRepository,
};

pub struct HtpasswdAuthRepository {
    htpasswd_path: String,
}

impl HtpasswdAuthRepository {
    pub fn new(htpasswd_path: String) -> Self {
        Self { htpasswd_path }
    }
}

impl Default for HtpasswdAuthRepository {
    fn default() -> Self {
        Self::new(".htpasswd".to_string())
    }
}

#[async_trait]
impl AuthRepository for HtpasswdAuthRepository {
    async fn authenticate(&self, username: &str, password: &str) -> Result<AuthUser, DomainError> {
        let htpasswd_path = self.htpasswd_path.clone();
        let username_str = username.to_string();
        let password_str = password.to_string();
        
        let is_valid = task::spawn_blocking(move || -> Result<bool, DomainError> {
            let htpasswd_content = fs::read_to_string(&htpasswd_path)
                .map_err(|e| {
                    error!("Failed to read htpasswd file: {}", e);
                    DomainError::AuthenticationFailed
                })?;
            
            let htpasswd = Htpasswd::from(htpasswd_content.as_str());
            Ok(htpasswd.check(&username_str, &password_str))
        })
        .await
        .map_err(|e| {
            error!("Authentication task failed: {}", e);
            DomainError::AuthenticationFailed
        })??;
        
        if is_valid {
            debug!("Authentication successful for user: {}", username);
            Ok(AuthUser::new(username.to_string()))
        } else {
            debug!("Authentication failed for user: {}", username);
            Err(DomainError::AuthenticationFailed)
        }
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<AuthUser>, DomainError> {
        let htpasswd_path = self.htpasswd_path.clone();
        let username_str = username.to_string();
        
        let user_exists = task::spawn_blocking(move || -> Result<bool, DomainError> {
            let htpasswd_content = fs::read_to_string(&htpasswd_path)
                .map_err(|_| DomainError::AuthenticationFailed)?;
            
            // Check if user exists by trying to find the username in the file
            Ok(htpasswd_content.lines()
                .any(|line| line.starts_with(&format!("{}:", username_str))))
        })
        .await
        .map_err(|_| DomainError::AuthenticationFailed)??;
        
        if user_exists {
            Ok(Some(AuthUser::new(username.to_string())))
        } else {
            Ok(None)
        }
    }
}
