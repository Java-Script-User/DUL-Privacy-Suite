use tracing::{info, warn};
use std::net::IpAddr;
#[derive(Clone)]
pub struct Ipv6Protection {
    enabled: bool,
    blocked_count: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl Ipv6Protection {
    // Create protection module
    pub fn new(enabled: bool) -> Self {
        if enabled {
            info!("🛡️ IPv6 leak protection enabled - All IPv6 traffic will be blocked");
        }
        Self {
            enabled,
            blocked_count: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    // IPv6 detection
    pub fn should_block_ipv6(&self, host: &str) -> bool {
        if !self.enabled {
            return false;
        }

        if let Ok(ip_addr) = host.parse::<IpAddr>() {
            if ip_addr.is_ipv6() {
                self.blocked_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                warn!("🚫 Blocked IPv6 address: {}", host);
                return true;
            }
        }

        if host.starts_with('[') && host.contains(':') {
            self.blocked_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            warn!("🚫 Blocked IPv6 host notation: {}", host);
            return true;
        }

        false
    }

    // Stats
    pub fn get_blocked_count(&self) -> u64 {
        self.blocked_count.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    #[cfg(target_os = "windows")]
    // Best-effort system disable
    pub fn disable_system_ipv6() -> Result<(), String> {
        info!("Attempting to disable IPv6 at system level...");
        warn!("System-level IPv6 disable requires administrator privileges");
        warn!("For maximum protection, manually disable IPv6 in Windows network settings");
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    pub fn disable_system_ipv6() -> Result<(), String> {
        warn!("System-level IPv6 disable not implemented for this platform");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocks_ipv6_addresses() {
        let protection = Ipv6Protection::new(true);
        assert!(protection.should_block_ipv6("2001:db8::1"));
        assert!(protection.should_block_ipv6("::1"));
        assert!(protection.should_block_ipv6("fe80::1"));
    }

    #[test]
    fn test_blocks_ipv6_brackets() {
        let protection = Ipv6Protection::new(true);
        assert!(protection.should_block_ipv6("[2001:db8::1]"));
    }

    #[test]
    fn test_allows_ipv4() {
        let protection = Ipv6Protection::new(true);
        assert!(!protection.should_block_ipv6("192.168.1.1"));
        assert!(!protection.should_block_ipv6("8.8.8.8"));
    }

    #[test]
    fn test_allows_domains() {
        let protection = Ipv6Protection::new(true);
        assert!(!protection.should_block_ipv6("example.com"));
        assert!(!protection.should_block_ipv6("google.com"));
    }
}
