use std::sync::Arc;
use std::time::Duration;
use tokio::time::{interval, Interval};
use log::{info, error, debug};

use crate::usecase::VpnUserUseCase;

pub struct VpnUserScheduler {
    use_case: Arc<VpnUserUseCase>,
    interval: Interval,
}

impl VpnUserScheduler {
    pub fn new(use_case: Arc<VpnUserUseCase>, interval_seconds: u64) -> Self {
        let interval = interval(Duration::from_secs(interval_seconds));
        Self { use_case, interval }
    }

    pub async fn start(&mut self) {
        info!("Starting VPN user scheduler");
        
        loop {
            self.interval.tick().await;
            
            match self.use_case.fetch_and_update_users().await {
                Ok(users) => {
                    debug!("Scheduled update completed: {} users", users.len());
                }
                Err(e) => {
                    error!("Scheduled update failed: {}", e);
                }
            }
        }
    }
}

#[allow(dead_code)]
pub struct SchedulerService {
    vpn_user_scheduler: Option<VpnUserScheduler>,
}

#[allow(dead_code)]
impl SchedulerService {
    pub fn new() -> Self {
        Self {
            vpn_user_scheduler: None,
        }
    }

    pub fn with_vpn_user_scheduler(mut self, scheduler: VpnUserScheduler) -> Self {
        self.vpn_user_scheduler = Some(scheduler);
        self
    }

    pub async fn start_all(&mut self) {
        if let Some(mut scheduler) = self.vpn_user_scheduler.take() {
            tokio::spawn(async move {
                scheduler.start().await;
            });
        }
    }
}

impl Default for SchedulerService {
    fn default() -> Self {
        Self::new()
    }
}
