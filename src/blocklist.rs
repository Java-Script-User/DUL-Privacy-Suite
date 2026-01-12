use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use tracing::info;

#[derive(Clone)]
pub struct TrackerBlocker {
    blocked_domains: HashSet<String>,
    blocked_count: Arc<Mutex<u64>>,
}

impl TrackerBlocker {
    pub fn new() -> Self {
        let mut blocked_domains = HashSet::new();
        
        // Common tracking and ad domains (comprehensive list)
        let trackers = vec![
            // Google Analytics & Ads
            "google-analytics.com",
            "googletagmanager.com",
            "doubleclick.net",
            "googlesyndication.com",
            "googleadservices.com",
            "2mdn.net",
            "googletagservices.com",
            "google.com/ads",
            "google.com/pagead",
            
            // Facebook tracking
            "facebook.com/tr",
            "facebook.net",
            "connect.facebook.net",
            "fbcdn.net",
            "facebook.com/plugins",
            
            // Twitter/X tracking
            "analytics.twitter.com",
            "ads-twitter.com",
            "ads-api.twitter.com",
            "static.ads-twitter.com",
            
            // LinkedIn tracking
            "ads.linkedin.com",
            "px.ads.linkedin.com",
            "analytics.pointdrive.linkedin.com",
            
            // TikTok tracking
            "analytics.tiktok.com",
            "ads.tiktok.com",
            
            // Major analytics platforms
            "scorecardresearch.com",
            "quantserve.com",
            "omtrdc.net",
            "demdex.net",
            "2o7.net",
            "chartbeat.com",
            "chartbeat.net",
            "hotjar.com",
            "mouseflow.com",
            "crazyegg.com",
            "fullstory.com",
            
            // Microsoft tracking
            "clarity.ms",
            "bing.com/fd",
            "bat.bing.com",
            
            // Amazon tracking
            "amazon-adsystem.com",
            "assoc-amazon.com",
            
            // Major ad networks
            "advertising.com",
            "adnxs.com",
            "pubmatic.com",
            "rubiconproject.com",
            "openx.net",
            "casalemedia.com",
            "criteo.com",
            "criteo.net",
            "bidswitch.net",
            "taboola.com",
            "outbrain.com",
            "smartadserver.com",
            "adform.net",
            "serving-sys.com",
            "mathtag.com",
            "adsrvr.org",
            "bluekai.com",
            "krxd.net",
            "exelator.com",
            "mookie1.com",
            "addthis.com",
            "sharethis.com",
            
            // Tracking pixels
            "pixel.facebook.com",
            "analytics.google.com",
            "stats.g.doubleclick.net",
            "pagead2.googlesyndication.com",
            
            // CDNs used primarily for tracking
            "cdn.segment.com",
            "cdn.segment.io",
            "api.segment.io",
            
            // Other major trackers
            "mixpanel.com",
            "amplitude.com",
            "heap.io",
            "loggly.com",
            "bugsnag.com",
            "sentry.io",
        ];
        
        for tracker in trackers {
            blocked_domains.insert(tracker.to_string());
        }
        
        info!("Loaded {} tracking domains to block", blocked_domains.len());
        
        Self { 
            blocked_domains,
            blocked_count: Arc::new(Mutex::new(0)),
        }
    }
    
    /// Check if a domain should be blocked
    pub fn should_block(&self, domain: &str) -> bool {
        let should_block = {
            // Check exact match
            if self.blocked_domains.contains(domain) {
                true
            } else {
                // Check if any parent domain matches (e.g., sub.google-analytics.com matches google-analytics.com)
                let parts: Vec<&str> = domain.split('.').collect();
                let mut found = false;
                for i in 0..parts.len() {
                    let subdomain = parts[i..].join(".");
                    if self.blocked_domains.contains(&subdomain) {
                        found = true;
                        break;
                    }
                }
                
                // Also check if domain contains common tracking patterns
                if !found {
                    let lower_domain = domain.to_lowercase();
                    found = lower_domain.contains("/tr") || 
                            lower_domain.contains("analytics") ||
                            lower_domain.contains("/ads") ||
                            lower_domain.contains("doubleclick") ||
                            lower_domain.contains("tracking") ||
                            lower_domain.contains("pixel");
                }
                
                found
            }
        };
        
        if should_block {
            if let Ok(mut count) = self.blocked_count.lock() {
                *count += 1;
            }
        }
        
        should_block
    }
    
    /// Get total number of domains in blocklist
    pub fn blocklist_size(&self) -> usize {
        self.blocked_domains.len()
    }
    
    /// Get total number of trackers blocked this session
    pub fn total_blocked(&self) -> u64 {
        self.blocked_count.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_blocking() {
        let blocker = TrackerBlocker::new();
        
        // Should block tracking domains
        assert!(blocker.should_block("google-analytics.com"));
        assert!(blocker.should_block("www.google-analytics.com"));
        assert!(blocker.should_block("stats.google-analytics.com"));
        
        // Should not block normal domains
        assert!(!blocker.should_block("google.com"));
        assert!(!blocker.should_block("example.com"));
    }
}
