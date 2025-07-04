use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::{
    models::{DomainError, VpnUser},
    traits::CacheService,
};

pub struct InMemoryCache {
    vpn_users: Arc<RwLock<HashMap<String, VpnUser>>>,
    latencies: Arc<RwLock<HashMap<String, Option<f64>>>>,
}

impl InMemoryCache {
    pub fn new() -> Self {
        Self {
            vpn_users: Arc::new(RwLock::new(HashMap::new())),
            latencies: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryCache {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CacheService for InMemoryCache {
    async fn get_vpn_users(&self) -> Result<Option<Vec<VpnUser>>, DomainError> {
        let users = self.vpn_users.read().await;

        if users.is_empty() {
            Ok(None)
        } else {
            let users_vec: Vec<VpnUser> = users.values().cloned().collect();
            Ok(Some(users_vec))
        }
    }

    async fn set_vpn_users(&self, users: Vec<VpnUser>) -> Result<(), DomainError> {
        let mut cache = self.vpn_users.write().await;
        cache.clear();

        for user in users {
            cache.insert(user.name.clone(), user);
        }

        Ok(())
    }

    async fn get_user_latency(&self, user_name: &str) -> Result<Option<f64>, DomainError> {
        let latencies = self.latencies.read().await;
        Ok(latencies.get(user_name).cloned().flatten())
    }

    async fn set_user_latency(&self, user_name: &str, latency: Option<f64>) -> Result<(), DomainError> {
        let mut latencies = self.latencies.write().await;
        latencies.insert(user_name.to_string(), latency);

        // Also update the user's latency in the users cache
        let mut users = self.vpn_users.write().await;
        if let Some(user) = users.get_mut(user_name) {
            user.latency = latency;
        }

        Ok(())
    }

    async fn clear_user(&self, user_name: &str) -> Result<(), DomainError> {
        let mut users = self.vpn_users.write().await;
        users.remove(user_name);

        let mut latencies = self.latencies.write().await;
        latencies.remove(user_name);

        Ok(())
    }
}
