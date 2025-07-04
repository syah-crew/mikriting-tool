use crate::domain::{
    models::{VpnUser, LatencyUpdate, DomainError},
    traits::{VpnUserRepository, MikrotikService, PingService, EventPublisher, CacheService}
};
use std::sync::Arc;
use log::{info, error, debug};

pub struct VpnUserUseCase {
    vpn_user_repository: Arc<dyn VpnUserRepository + Send + Sync>,
    mikrotik_service: Arc<dyn MikrotikService + Send + Sync>,
    ping_service: Arc<dyn PingService + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    cache_service: Arc<dyn CacheService + Send + Sync>,
}

impl VpnUserUseCase {
    pub fn new(
        vpn_user_repository: Arc<dyn VpnUserRepository + Send + Sync>,
        mikrotik_service: Arc<dyn MikrotikService + Send + Sync>,
        ping_service: Arc<dyn PingService + Send + Sync>,
        event_publisher: Arc<dyn EventPublisher + Send + Sync>,
        cache_service: Arc<dyn CacheService + Send + Sync>,
    ) -> Self {
        Self {
            vpn_user_repository,
            mikrotik_service,
            ping_service,
            event_publisher,
            cache_service,
        }
    }

    pub async fn fetch_and_update_users(&self) -> Result<Vec<VpnUser>, DomainError> {
        debug!("Fetching VPN users from MikroTik");
        
        // Fetch from MikroTik
        let fresh_users = self.mikrotik_service.fetch_active_connections().await?;
        
        // Get cached users for comparison
        let cached_users = self.cache_service.get_vpn_users().await?
            .unwrap_or_default();
        
        // Process new and disconnected users
        self.process_user_changes(&cached_users, &fresh_users).await?;
        
        // Update cache
        self.cache_service.set_vpn_users(fresh_users.clone()).await?;
        
        // Publish update event
        self.event_publisher.publish_vpn_users_update(fresh_users.clone()).await?;
        
        info!("Updated {} VPN users", fresh_users.len());
        Ok(fresh_users)
    }

    async fn process_user_changes(
        &self,
        old_users: &[VpnUser],
        new_users: &[VpnUser],
    ) -> Result<(), DomainError> {
        let old_names: std::collections::HashSet<_> = old_users.iter()
            .map(|u| u.name.clone())
            .collect();
        
        let new_names: std::collections::HashSet<_> = new_users.iter()
            .map(|u| u.name.clone())
            .collect();

        // Handle disconnected users
        for disconnected_user in old_names.difference(&new_names) {
            debug!("User disconnected: {}", disconnected_user);
            self.ping_service.stop_monitoring(disconnected_user).await?;
            self.cache_service.clear_user(disconnected_user).await?;
        }

        // Handle new users
        for new_user in new_users.iter().filter(|u| !old_names.contains(&u.name)) {
            debug!("New user connected: {}", new_user.name);
            self.ping_service.start_monitoring(new_user).await?;
        }

        Ok(())
    }

    pub async fn get_all_users(&self) -> Result<Vec<VpnUser>, DomainError> {
        // Try cache first
        if let Some(cached_users) = self.cache_service.get_vpn_users().await? {
            return Ok(cached_users);
        }

        // Fallback to repository
        self.vpn_user_repository.find_all().await
    }

    pub async fn update_user_latency(&self, update: LatencyUpdate) -> Result<(), DomainError> {
        debug!("Updating latency for user: {} -> {:?}", update.user_name, update.latency);
        
        // Update cache
        self.cache_service.set_user_latency(&update.user_name, update.latency).await?;
        
        // Publish latency update
        self.event_publisher.publish_latency_update(update).await?;
        
        Ok(())
    }

    pub async fn disconnect_user(&self, user_name: &str) -> Result<(), DomainError> {
        info!("Disconnecting user: {}", user_name);
        
        // Disconnect from MikroTik
        self.mikrotik_service.disconnect_user(user_name).await?;
        
        // Stop monitoring
        self.ping_service.stop_monitoring(user_name).await?;
        
        // Clear from cache
        self.cache_service.clear_user(user_name).await?;
        
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn get_user_by_name(&self, name: &str) -> Result<Option<VpnUser>, DomainError> {
        self.vpn_user_repository.find_by_name(name).await
    }
}

pub struct AuthUseCase {
    auth_repository: Arc<dyn crate::domain::traits::AuthRepository + Send + Sync>,
}

impl AuthUseCase {
    pub fn new(auth_repository: Arc<dyn crate::domain::traits::AuthRepository + Send + Sync>) -> Self {
        Self { auth_repository }
    }

    pub async fn authenticate(&self, username: &str, password: &str) -> Result<crate::domain::models::AuthUser, DomainError> {
        debug!("Attempting authentication for user: {}", username);
        
        let auth_user = self.auth_repository.authenticate(username, password).await?;
        
        if auth_user.is_authenticated {
            info!("User authenticated successfully: {}", username);
        } else {
            error!("Authentication failed for user: {}", username);
        }
        
        Ok(auth_user)
    }

    #[allow(dead_code)]
    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<crate::domain::models::AuthUser>, DomainError> {
        self.auth_repository.find_by_username(username).await
    }
}
