use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::{
    models::{VpnUser, DomainError},
    traits::VpnUserRepository,
};

pub struct InMemoryVpnUserRepository {
    users: Arc<RwLock<HashMap<String, VpnUser>>>,
}

impl InMemoryVpnUserRepository {
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryVpnUserRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VpnUserRepository for InMemoryVpnUserRepository {
    async fn find_all(&self) -> Result<Vec<VpnUser>, DomainError> {
        let users = self.users.read().await;
        Ok(users.values().cloned().collect())
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<VpnUser>, DomainError> {
        let users = self.users.read().await;
        Ok(users.get(name).cloned())
    }

    async fn save(&self, user: &VpnUser) -> Result<(), DomainError> {
        let mut users = self.users.write().await;
        users.insert(user.name.clone(), user.clone());
        Ok(())
    }

    async fn delete(&self, name: &str) -> Result<(), DomainError> {
        let mut users = self.users.write().await;
        if users.remove(name).is_some() {
            Ok(())
        } else {
            Err(DomainError::UserNotFound(name.to_string()))
        }
    }

    async fn update_latency(&self, name: &str, latency: Option<f64>) -> Result<(), DomainError> {
        let mut users = self.users.write().await;
        
        if let Some(user) = users.get_mut(name) {
            user.latency = latency;
            Ok(())
        } else {
            Err(DomainError::UserNotFound(name.to_string()))
        }
    }
}
