mod domain;
mod usecase;
mod adapter;
mod infrastructure;

use std::sync::Arc;
use log::info;
use env_logger;
use actix::Actor;

use crate::domain::traits::*;
use crate::usecase::*;
use crate::adapter::*;
use crate::infrastructure::*;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    info!("Starting mikriting-tool application");
    
    // Create configuration service
    let config_service = Arc::new(FileConfigService::new()) as Arc<dyn ConfigService + Send + Sync>;
    
    // Load configurations
    let app_config = config_service.get_app_config()
        .expect("Failed to load app configuration");
    let mikrotik_config = config_service.get_mikrotik_config()
        .expect("Failed to load MikroTik configuration");
    
    info!("Configuration loaded successfully");
    
    // Create infrastructure services
    let vpn_user_repository = Arc::new(InMemoryVpnUserRepository::new()) as Arc<dyn VpnUserRepository + Send + Sync>;
    let auth_repository = Arc::new(HtpasswdAuthRepository::default()) as Arc<dyn AuthRepository + Send + Sync>;
    let cache_service = Arc::new(InMemoryCache::new()) as Arc<dyn CacheService + Send + Sync>;
    
    // Create MikroTik client
    let mikrotik_service = Arc::new(
        MikrotikClient::new(mikrotik_config)
            .expect("Failed to create MikroTik client")
    ) as Arc<dyn MikrotikService + Send + Sync>;
    
    // Create WebSocket manager and event publisher
    let websocket_manager = WebSocketManager::new().start();
    let event_publisher = Arc::new(WebSocketEventPublisher::new(websocket_manager.clone())) as Arc<dyn EventPublisher + Send + Sync>;
    
    // Create ping service
    let ping_service = Arc::new(
        PingMonitor::new(
            // We'll create a temp use case for ping monitor
            Arc::new(VpnUserUseCase::new(
                vpn_user_repository.clone(),
                mikrotik_service.clone(),
                Arc::new(DummyPingService::new()) as Arc<dyn PingService + Send + Sync>,
                event_publisher.clone(),
                cache_service.clone(),
            )),
            app_config.ping_interval_seconds
        )
        .await
        .expect("Failed to create ping monitor")
    ) as Arc<dyn PingService + Send + Sync>;
    
    // Create use cases with real ping service
    let vpn_user_use_case = Arc::new(VpnUserUseCase::new(
        vpn_user_repository,
        mikrotik_service,
        ping_service,
        event_publisher,
        cache_service,
    ));
    
    let auth_use_case = Arc::new(AuthUseCase::new(auth_repository));
    
    // Create and start scheduler
    let scheduler = VpnUserScheduler::new(vpn_user_use_case.clone(), 15); // 15 seconds interval
    tokio::spawn(async move {
        let mut scheduler = scheduler;
        scheduler.start().await;
    });
    
    info!("Background services started");
    
    // Start the web server
    adapter::rest_api::start_server(
        vpn_user_use_case,
        auth_use_case,
        config_service,
    ).await
}

// Dummy ping service for initial setup
struct DummyPingService;

impl DummyPingService {
    fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl PingService for DummyPingService {
    async fn ping_user(&self, _user: &domain::models::VpnUser) -> Result<Option<f64>, domain::models::DomainError> {
        Ok(Some(0.0))
    }
    
    async fn start_monitoring(&self, _user: &domain::models::VpnUser) -> Result<(), domain::models::DomainError> {
        Ok(())
    }
    
    async fn stop_monitoring(&self, _user_name: &str) -> Result<(), domain::models::DomainError> {
        Ok(())
    }
}
