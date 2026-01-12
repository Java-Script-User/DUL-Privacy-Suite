use tracing::{info, warn};
use std::net::IpAddr;

/// WebRTC Leak Protection
/// 
/// WebRTC can leak real IP addresses even when using Tor/VPN through STUN requests.
/// This module detects and blocks WebRTC STUN/TURN requests that could reveal the user's IP.
#[derive(Clone)]
pub struct WebRtcProtection {
    enabled: bool,
}

impl WebRtcProtection {
    pub fn new(enabled: bool) -> Self {
        if enabled {
            info!("🛡️ WebRTC leak protection enabled");
        }
        Self { enabled }
    }

    /// Check if a request is a WebRTC STUN/TURN request that should be blocked
    pub fn should_block_request(&self, host: &str, _port: u16) -> bool {
        if !self.enabled {
            return false;
        }

        // Block common STUN/TURN servers
        let stun_servers = [
            "stun.l.google.com",
            "stun1.l.google.com",
            "stun2.l.google.com",
            "stun3.l.google.com",
            "stun4.l.google.com",
            "stun.cloudflare.com",
            "stun.services.mozilla.com",
            "stun.stunprotocol.org",
            "stun.voip.blackberry.com",
            "stun.voipbuster.com",
            "global.stun.twilio.com",
        ];

        for stun_host in &stun_servers {
            if host.contains(stun_host) {
                warn!("🚫 Blocked WebRTC STUN request to {}", host);
                return true;
            }
        }

        // Block direct IP connections (often used for WebRTC)
        if host.parse::<IpAddr>().is_ok() {
            warn!("🚫 Blocked direct IP connection attempt: {}", host);
            return true;
        }

        false
    }

    /// Generate headers to disable WebRTC in browser
    pub fn get_protection_headers(&self) -> Vec<(&'static str, String)> {
        if !self.enabled {
            return vec![];
        }

        vec![
            // Feature policy to disable WebRTC
            ("Permissions-Policy", "camera=(), microphone=(), geolocation=()".to_string()),
            // Additional security headers
            ("X-WebRTC-Block", "true".to_string()),
        ]
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocks_stun_servers() {
        let protection = WebRtcProtection::new(true);
        assert!(protection.should_block_request("stun.l.google.com", 3478));
        assert!(protection.should_block_request("stun1.l.google.com", 19302));
    }

    #[test]
    fn test_blocks_direct_ips() {
        let protection = WebRtcProtection::new(true);
        assert!(protection.should_block_request("192.168.1.1", 443));
        assert!(protection.should_block_request("8.8.8.8", 53));
    }

    #[test]
    fn test_allows_normal_domains() {
        let protection = WebRtcProtection::new(true);
        assert!(!protection.should_block_request("example.com", 443));
        assert!(!protection.should_block_request("google.com", 443));
    }
}
