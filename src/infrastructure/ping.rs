use async_trait::async_trait;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::interval;
use tokio_icmp_echo::Pinger;
use log::{debug, error, info, warn};

use crate::domain::{
    models::{VpnUser, LatencyUpdate, DomainError},
    traits::PingService,
};
use crate::usecase::VpnUserUseCase;

pub struct PingMonitor {
    pinger: Pinger,
    monitored_users: Arc<RwLock<HashMap<String, VpnUser>>>,
    use_case: Arc<VpnUserUseCase>,
    ping_interval: Duration,
}

impl PingMonitor {
    pub async fn new(
        use_case: Arc<VpnUserUseCase>,
        ping_interval_seconds: u64,
    ) -> Result<Self, DomainError> {
        let pinger = Pinger::new()
            .await
            .map_err(|e| DomainError::NetworkError(format!("Failed to create pinger: {}", e)))?;
        
        Ok(Self {
            pinger,
            monitored_users: Arc::new(RwLock::new(HashMap::new())),
            use_case,
            ping_interval: Duration::from_secs(ping_interval_seconds),
        })
    }

    #[allow(dead_code)]
    pub async fn start_monitoring_loop(&self) {
        let mut interval = interval(self.ping_interval);
        
        loop {
            interval.tick().await;
            
            let users = {
                let monitored = self.monitored_users.read().await;
                monitored.values().cloned().collect::<Vec<_>>()
            };
            
            if users.is_empty() {
                continue;
            }
            
            debug!("Pinging {} users", users.len());
            
            for user in users {
                if let Some(ip_addr) = user.get_ip_address() {
                    let latency = self.ping_single_user(&ip_addr).await;
                    
                    let update = LatencyUpdate {
                        user_name: user.name.clone(),
                        latency,
                    };
                    
                    if let Err(e) = self.use_case.update_user_latency(update).await {
                        error!("Failed to update latency for user {}: {}", user.name, e);
                    }
                }
            }
        }
    }

    async fn ping_single_user(&self, ip_addr: &IpAddr) -> Option<f64> {
        let ident = rand::random();
        let seq = rand::random();
        let timeout = Duration::from_secs(1);
        
        match self.pinger.ping(*ip_addr, ident, seq, timeout).await {
            Ok(Some(duration)) => {
                let latency_ms = duration.as_micros() as f64 / 1000.0;
                Some(latency_ms)
            }
            Ok(None) => {
                debug!("Ping timeout for {}", ip_addr);
                None
            }
            Err(e) => {
                debug!("Ping error for {}: {}", ip_addr, e);
                None
            }
        }
    }
}

#[async_trait]
impl PingService for PingMonitor {
    async fn ping_user(&self, user: &VpnUser) -> Result<Option<f64>, DomainError> {
        if let Some(ip_addr) = user.get_ip_address() {
            Ok(self.ping_single_user(&ip_addr).await)
        } else {
            Err(DomainError::InvalidIpAddress(user.address.clone()))
        }
    }

    async fn start_monitoring(&self, user: &VpnUser) -> Result<(), DomainError> {
        if user.get_ip_address().is_none() {
            return Err(DomainError::InvalidIpAddress(user.address.clone()));
        }
        
        let mut monitored = self.monitored_users.write().await;
        monitored.insert(user.name.clone(), user.clone());
        
        info!("Started monitoring user: {} ({})", user.name, user.address);
        Ok(())
    }

    async fn stop_monitoring(&self, user_name: &str) -> Result<(), DomainError> {
        let mut monitored = self.monitored_users.write().await;
        
        if monitored.remove(user_name).is_some() {
            info!("Stopped monitoring user: {}", user_name);
        } else {
            warn!("Attempted to stop monitoring non-existent user: {}", user_name);
        }
        
        Ok(())
    }
}
