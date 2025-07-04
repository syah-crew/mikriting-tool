use async_trait::async_trait;
use crate::domain::models::{VpnUser, AuthUser, LatencyUpdate, DomainError};

// Repository traits for data persistence
#[allow(dead_code)]
#[async_trait]
pub trait VpnUserRepository {
    async fn find_all(&self) -> Result<Vec<VpnUser>, DomainError>;
    async fn find_by_name(&self, name: &str) -> Result<Option<VpnUser>, DomainError>;
    async fn save(&self, user: &VpnUser) -> Result<(), DomainError>;
    async fn delete(&self, name: &str) -> Result<(), DomainError>;
    async fn update_latency(&self, name: &str, latency: Option<f64>) -> Result<(), DomainError>;
}

#[allow(dead_code)]
#[async_trait]
pub trait AuthRepository {
    async fn authenticate(&self, username: &str, password: &str) -> Result<AuthUser, DomainError>;
    async fn find_by_username(&self, username: &str) -> Result<Option<AuthUser>, DomainError>;
}

// External service interfaces
#[allow(dead_code)]
#[async_trait]
pub trait MikrotikService {
    async fn fetch_active_connections(&self) -> Result<Vec<VpnUser>, DomainError>;
    async fn disconnect_user(&self, user_name: &str) -> Result<(), DomainError>;
}

#[allow(dead_code)]
#[async_trait]
pub trait PingService {
    async fn ping_user(&self, user: &VpnUser) -> Result<Option<f64>, DomainError>;
    async fn start_monitoring(&self, user: &VpnUser) -> Result<(), DomainError>;
    async fn stop_monitoring(&self, user_name: &str) -> Result<(), DomainError>;
}

// Event handling
#[async_trait]
pub trait EventPublisher {
    async fn publish_vpn_users_update(&self, users: Vec<VpnUser>) -> Result<(), DomainError>;
    async fn publish_latency_update(&self, update: LatencyUpdate) -> Result<(), DomainError>;
}

// Cache interface
#[allow(dead_code)]
#[async_trait]
pub trait CacheService {
    async fn get_vpn_users(&self) -> Result<Option<Vec<VpnUser>>, DomainError>;
    async fn set_vpn_users(&self, users: Vec<VpnUser>) -> Result<(), DomainError>;
    async fn get_user_latency(&self, user_name: &str) -> Result<Option<f64>, DomainError>;
    async fn set_user_latency(&self, user_name: &str, latency: Option<f64>) -> Result<(), DomainError>;
    async fn clear_user(&self, user_name: &str) -> Result<(), DomainError>;
}

// Configuration interface
pub trait ConfigService {
    fn get_mikrotik_config(&self) -> Result<crate::domain::models::MikrotikConfig, DomainError>;
    fn get_app_config(&self) -> Result<crate::domain::models::AppConfig, DomainError>;
}
