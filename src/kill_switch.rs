use tracing::{info, warn};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Kill Switch - Blocks all traffic if Tor connection fails
/// 
/// This prevents IP leaks when the Tor network disconnects.
/// In a production app, this would integrate with OS firewall rules.
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

    /// Set Tor connection status
    pub async fn set_tor_status(&self, connected: bool) {
        let mut state = self.state.write().await;
        state.tor_connected = connected;
        
        if connected {
            info!("✅ Kill switch: Tor connected, allowing traffic");
        } else {
            warn!("⚠️ Kill switch: Tor disconnected, BLOCKING all traffic");
        }
    }

    /// Check if traffic should be allowed
    pub async fn should_allow_traffic(&self) -> bool {
        let mut state = self.state.write().await;
        
        if !state.kill_switch_active {
            return true; // Kill switch disabled
        }

        if !state.tor_connected {
            state.blocked_requests += 1;
            warn!("🚫 Kill switch: Blocked request (Tor not connected) - Total blocked: {}", state.blocked_requests);
            return false;
        }

        true
    }

    /// Enable or disable kill switch
    pub async fn set_enabled(&self, enabled: bool) {
        let mut state = self.state.write().await;
        state.kill_switch_active = enabled;
        
        if enabled {
            info!("🔒 Kill switch ENABLED - Will block traffic if Tor disconnects");
        } else {
            warn!("⚠️ Kill switch DISABLED - Traffic may leak if Tor fails!");
        }
    }

    /// Get current statistics
    pub async fn get_stats(&self) -> KillSwitchStats {
        let state = self.state.read().await;
        KillSwitchStats {
            tor_connected: state.tor_connected,
            active: state.kill_switch_active,
            blocked_requests: state.blocked_requests,
        }
    }

    /// Check if Tor is connected
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
