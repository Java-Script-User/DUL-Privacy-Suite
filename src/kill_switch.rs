use tracing::{info, warn};
use std::sync::Arc;
use tokio::sync::RwLock;
#[derive(Clone)]
pub struct KillSwitch {
    state: Arc<RwLock<KillSwitchState>>,
}

#[derive(Debug, Clone)]
struct KillSwitchState {
    tor_connected: bool,
    kill_switch_active: bool,
    blocked_requests: u64,
}

impl KillSwitch {
    // Create kill switch
    pub fn new() -> Self {
        info!("🔒 Kill switch initialized");
        Self {
            state: Arc::new(RwLock::new(KillSwitchState {
                tor_connected: false,
                kill_switch_active: true,
                blocked_requests: 0,
            })),
        }
    }

    // Update Tor state
    pub async fn set_tor_status(&self, connected: bool) {
        let mut state = self.state.write().await;
        state.tor_connected = connected;
        
        if connected {
            info!("✅ Kill switch: Tor connected, allowing traffic");
        } else {
            warn!("⚠️ Kill switch: Tor disconnected, BLOCKING all traffic");
        }
    }

    // Gate traffic based on Tor status
    pub async fn should_allow_traffic(&self) -> bool {
        let mut state = self.state.write().await;
        
        if !state.kill_switch_active {
            return true;
        }

        if !state.tor_connected {
            state.blocked_requests += 1;
            warn!("🚫 Kill switch: Blocked request (Tor not connected) - Total blocked: {}", state.blocked_requests);
            return false;
        }

        true
    }

    // Enable or disable
    pub async fn set_enabled(&self, enabled: bool) {
        let mut state = self.state.write().await;
        state.kill_switch_active = enabled;
        
        if enabled {
            info!("🔒 Kill switch ENABLED - Will block traffic if Tor disconnects");
        } else {
            warn!("⚠️ Kill switch DISABLED - Traffic may leak if Tor fails!");
        }
    }

    // Read stats
    pub async fn get_stats(&self) -> KillSwitchStats {
        let state = self.state.read().await;
        KillSwitchStats {
            tor_connected: state.tor_connected,
            active: state.kill_switch_active,
            blocked_requests: state.blocked_requests,
        }
    }

    pub async fn is_tor_connected(&self) -> bool {
        let state = self.state.read().await;
        state.tor_connected
    }
}

#[derive(Debug, Clone)]
pub struct KillSwitchStats {
    pub tor_connected: bool,
    pub active: bool,
    pub blocked_requests: u64,
}

impl Default for KillSwitch {
    fn default() -> Self {
        Self::new()
    }
}
