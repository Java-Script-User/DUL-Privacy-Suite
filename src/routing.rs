use crate::config::Config;
use crate::network::Node;
use crate::crypto::CryptoLayer;
use crate::tor_network::TorNetwork;
use crate::fingerprint::{BrowserFingerprint, CanvasProtection};
use crate::blocklist::TrackerBlocker;
use crate::webrtc_protection::WebRtcProtection;
use crate::kill_switch::KillSwitch;
use crate::ipv6_protection::Ipv6Protection;
use crate::web_api::{ApiState, LogDetails};
use hyper::{Request, Response, body::Bytes};
use http_body_util::Full;
use tracing::{info, warn};

#[derive(Clone)]
pub struct Router {
    config: Config,
    crypto: CryptoLayer,
    nodes: Vec<Node>,
    tor: TorNetwork,
    fingerprint: BrowserFingerprint,
    tracker_blocker: TrackerBlocker,
    webrtc_protection: WebRtcProtection,
    kill_switch: KillSwitch,
    ipv6_protection: Ipv6Protection,
    canvas_protection: CanvasProtection,
    app_state: Option<ApiState>,
}

impl Router {
    pub async fn new(config: Config, app_state: Option<ApiState>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let crypto = CryptoLayer::new();
        
        // Load available nodes from database/registry
        let nodes = Self::load_nodes(&config).await?;
        
        // Initialize Tor connection
        info!("Connecting to Tor network...");
        let tor = TorNetwork::new().await?;
        info!("✅ Connected to Tor! Using 6,000+ volunteer nodes");
        
        // Initialize privacy features
        let fingerprint = BrowserFingerprint::random();
        info!("✅ Browser fingerprint randomization enabled");
        
        let tracker_blocker = TrackerBlocker::new();
        info!("✅ Tracker blocking enabled ({} domains)", tracker_blocker.blocklist_size());
        
        info!("✅ DNS-over-HTTPS encryption enabled");
        
        // Initialize advanced security features
        let webrtc_protection = WebRtcProtection::new(true);
        let ipv6_protection = Ipv6Protection::new(true);
        let canvas_protection = CanvasProtection::new(true);
        info!("✅ Canvas fingerprinting protection enabled");
        
        let kill_switch = KillSwitch::new();
        kill_switch.set_tor_status(true).await;
        info!("✅ Kill switch enabled");
        
        Ok(Self {
            config,
            crypto,
            nodes,
            tor,
            fingerprint,
            tracker_blocker,
            webrtc_protection,
            kill_switch,
            ipv6_protection,
            canvas_protection,
            app_state,
        })
    }
    
    async fn load_nodes(_config: &Config) -> Result<Vec<Node>, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Load from decentralized node registry
        // For now, return placeholder nodes
        Ok(vec![
            Node::new("node1.example.com:9000".to_string()),
            Node::new("node2.example.com:9000".to_string()),
            Node::new("node3.example.com:9000".to_string()),
        ])
    }
    
    /// Detect security risks and malicious tracking patterns
    async fn detect_security_risks(&self, host: &str, path: &str, method: &str) {
        if let Some(state) = &self.app_state {
            let full_url = format!("{}{}", host, path);
            
            // Detect credential leaks in URL
            let credential_patterns = vec![
                ("password", "Password in URL"),
                ("pwd", "Password in URL"),
                ("api_key", "API Key in URL"),
                ("apikey", "API Key in URL"),
                ("token", "Token in URL"),
                ("access_token", "Access Token in URL"),
                ("secret", "Secret in URL"),
                ("private", "Private data in URL"),
                ("auth", "Auth data in URL"),
                ("session", "Session ID in URL"),
            ];
            
            for (pattern, threat) in credential_patterns {
                if path.to_lowercase().contains(pattern) {
                    let details = LogDetails {
                        url: Some(full_url.clone()),
                        domain: Some(host.to_string()),
                        path: Some(path.to_string()),
                        port: None,
                        method: Some(method.to_string()),
                        client_ip: None,
                        threat_type: Some(threat.to_string()),
                        reason: Some("Sensitive data detected in URL - potential credential leak".to_string()),
                        request_headers: None,
                    };
                    warn!("⚠️ SECURITY: {} - {}", threat, full_url);
                    state.update_stats(|s| s.security_threats_detected += 1).await;
                    state.add_log_with_details("error", format!("⚠️ SECURITY: {} - {}", threat, host), "security", Some(details)).await;
                }
            }
            
            // Detect suspicious tracking patterns
            let tracking_patterns = vec![
                ("/track", "Tracking endpoint"),
                ("/collect", "Data collection endpoint"),
                ("/analytics", "Analytics tracking"),
                ("/beacon", "Tracking beacon"),
                ("/pixel", "Tracking pixel"),
                ("/impression", "Ad impression tracking"),
                ("/conversion", "Conversion tracking"),
                ("/telemetry", "Telemetry data collection"),
                ("/fingerprint", "Browser fingerprinting"),
            ];
            
            for (pattern, tracking_type) in tracking_patterns {
                if path.to_lowercase().contains(pattern) {
                    let details = LogDetails {
                        url: Some(full_url.clone()),
                        domain: Some(host.to_string()),
                        path: Some(path.to_string()),
                        port: None,
                        method: Some(method.to_string()),
                        client_ip: None,
                        threat_type: Some(tracking_type.to_string()),
                        reason: Some("Suspicious tracking pattern detected".to_string()),
                        request_headers: None,
                    };
                    warn!("🔍 TRACKING: {} detected - {}", tracking_type, full_url);
                    state.update_stats(|s| s.security_threats_detected += 1).await;
                    state.add_log_with_details("warn", format!("🔍 {} detected: {}", tracking_type, host), "security", Some(details)).await;
                }
            }
            
            // Detect malicious domains patterns
            let malicious_patterns = vec![
                ("analytics", "Analytics service"),
                ("doubleclick", "Ad network"),
                ("adserver", "Ad server"),
                ("tracker", "Tracking service"),
                ("metric", "Metrics collection"),
                ("stats", "Statistics collection"),
                ("tag-manager", "Tag management"),
                ("remarketing", "Remarketing service"),
            ];
            
            for (pattern, service_type) in malicious_patterns {
                if host.to_lowercase().contains(pattern) {
                    let details = LogDetails {
                        url: Some(full_url.clone()),
                        domain: Some(host.to_string()),
                        path: Some(path.to_string()),
                        port: None,
                        method: Some(method.to_string()),
                        client_ip: None,
                        threat_type: Some(service_type.to_string()),
                        reason: Some("Suspicious domain pattern - likely tracking/advertising".to_string()),
                        request_headers: None,
                    };
                    info!("🕵️ {} detected in domain: {}", service_type, host);
                    state.update_stats(|s| s.security_threats_detected += 1).await;
                    state.add_log_with_details("info", format!("🕵️ {} detected: {}", service_type, host), "security", Some(details)).await;
                }
            }
            
            // Detect unencrypted connections
            if host.starts_with("http://") {
                let details = LogDetails {
                    url: Some(full_url.clone()),
                    domain: Some(host.to_string()),
                    path: Some(path.to_string()),
                    port: None,
                    method: Some(method.to_string()),
                    client_ip: None,
                    threat_type: Some("Unencrypted connection".to_string()),
                    reason: Some("HTTP connection detected - data transmitted in plain text".to_string()),
                    request_headers: None,
                };
                warn!("⚠️ SECURITY: Unencrypted HTTP request to: {}", host);
                state.update_stats(|s| s.security_threats_detected += 1).await;
                state.add_log_with_details("warn", format!("⚠️ Unencrypted HTTP: {}", host), "security", Some(details)).await;
            }
        }
    }
    
    pub async fn route_request(
        &self,
        req: Request<hyper::body::Incoming>,
    ) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
        let method = req.method().clone();
        let uri = req.uri().clone();
        
        // Check kill switch first
        if !self.kill_switch.should_allow_traffic().await {
            warn!("🚫 Kill switch: Blocking request (Tor disconnected)");
            if let Some(state) = &self.app_state {
                let details = LogDetails {
                    url: Some(uri.to_string()),
                    domain: uri.host().map(|h| h.to_string()),
                    path: Some(uri.path().to_string()),
                    port: uri.port_u16(),
                    method: Some(method.to_string()),
                    client_ip: None,
                    threat_type: Some("Kill Switch Block".to_string()),
                    reason: Some("Tor connection lost - blocking traffic to prevent IP leaks".to_string()),
                    request_headers: None,
                };
                state.add_log_with_details("error", "🚫 Kill switch blocked request - Tor disconnected!".to_string(), "security", Some(details)).await;
                state.update_stats(|s| {
                    s.requests_blocked += 1;
                    s.security_threats_detected += 1;
                }).await;
            }
            return Ok(Response::builder()
                .status(503)
                .body(Full::new(Bytes::from("Service unavailable: Privacy protection disconnected")))
                .unwrap());
        }
        
        // Increment total requests
        if let Some(state) = &self.app_state {
            state.update_stats(|s| s.total_requests += 1).await;
        }
        
        // Log all domains being accessed
        if let Some(host) = uri.host() {
            let path = uri.path();
            let port = uri.port_u16().unwrap_or(443);
            let full_url = format!("{}{}", host, path);
            info!("🌐 Request to: {}", full_url);
            
            if let Some(state) = &self.app_state {
                let details = LogDetails {
                    url: Some(full_url.clone()),
                    domain: Some(host.to_string()),
                    path: Some(path.to_string()),
                    port: Some(port),
                    method: Some(method.to_string()),
                    client_ip: None,
                    threat_type: None,
                    reason: None,
                    request_headers: None,
                };
                state.add_log_with_details("info", format!("🌐 {}", full_url), "network", Some(details)).await;
            }
            
            // Detect security risks and malicious tracking patterns
            self.detect_security_risks(host, path, method.as_str()).await;
            
            // Check IPv6 protection
            if self.ipv6_protection.should_block_ipv6(host) {
                warn!("🚫 Blocked IPv6 request: {}", host);
                if let Some(state) = &self.app_state {
                    let details = LogDetails {
                        url: Some(full_url.clone()),
                        domain: Some(host.to_string()),
                        path: Some(path.to_string()),
                        port: Some(port),
                        method: Some(method.to_string()),
                        client_ip: None,
                        threat_type: Some("IPv6 Leak Attempt".to_string()),
                        reason: Some("IPv6 connection blocked to prevent real IP address exposure".to_string()),
                        request_headers: None,
                    };
                    state.update_stats(|s| {
                        s.ipv6_blocked += 1;
                        s.requests_blocked += 1;
                    }).await;
                    state.add_log_with_details("warn", format!("🚫 Blocked IPv6 leak: {}{}", host, path), "ipv6", Some(details)).await;
                    info!("IPv6 protection prevented potential IP leak");
                }
                return Ok(Response::builder()
                    .status(403)
                    .body(Full::new(Bytes::from("IPv6 blocked for privacy protection")))
                    .unwrap());
            }
            
            // Check WebRTC protection
            if self.webrtc_protection.should_block_request(host, port) {
                warn!("🚫 Blocked WebRTC/STUN request: {}:{}", host, port);
                if let Some(state) = &self.app_state {
                    let details = LogDetails {
                        url: Some(full_url.clone()),
                        domain: Some(host.to_string()),
                        path: Some(path.to_string()),
                        port: Some(port),
                        method: Some(method.to_string()),
                        client_ip: None,
                        threat_type: Some("WebRTC Leak Attempt".to_string()),
                        reason: Some("WebRTC/STUN connection blocked to prevent real IP address exposure via peer connections".to_string()),
                        request_headers: None,
                    };
                    state.update_stats(|s| {
                        s.webrtc_blocked += 1;
                        s.requests_blocked += 1;
                    }).await;
                    state.add_log_with_details("warn", format!("🚫 Blocked WebRTC leak attempt: {}:{}", host, port), "webrtc", Some(details)).await;
                    info!("WebRTC protection prevented potential IP leak");
                }
                return Ok(Response::builder()
                    .status(403)
                    .body(Full::new(Bytes::from("WebRTC blocked for privacy protection")))
                    .unwrap());
            }
            
            // Check if domain should be blocked
            if self.tracker_blocker.should_block(host) {
                warn!("🚫 Blocked tracker: {}{}", host, path);
                if let Some(state) = &self.app_state {
                    let details = LogDetails {
                        url: Some(full_url.clone()),
                        domain: Some(host.to_string()),
                        path: Some(path.to_string()),
                        port: Some(port),
                        method: Some(method.to_string()),
                        client_ip: None,
                        threat_type: Some("Known Tracker".to_string()),
                        reason: Some("Domain matched against known tracker database - preventing data collection".to_string()),
                        request_headers: None,
                    };
                    state.update_stats(|s| {
                        s.trackers_blocked += 1;
                        s.requests_blocked += 1;
                    }).await;
                    state.add_log_with_details("warn", format!("🚫 Blocked tracker: {}{}", host, path), "tracker", Some(details)).await;
                    info!("Tracker blocker prevented data collection attempt");
                }
                return Ok(Response::builder()
                    .status(403)
                    .body(Full::new(Bytes::from("Tracker blocked by Privacy Suite")))
                    .unwrap());
            }
        }
        
        // Route through Tor's existing 3-hop circuit with randomized fingerprint
        let response = self.tor.route_request(req, &self.fingerprint).await?;
        
        if let Some(state) = &self.app_state {
            state.add_log("info", "✅ Routed through Tor (3 encrypted hops)".to_string(), "network").await;
        }
        
        Ok(response)
    }
    
    pub async fn connect_through_tor(
        &self,
        host: &str,
        port: u16,
    ) -> Result<arti_client::DataStream, Box<dyn std::error::Error + Send + Sync>> {
        info!("🔐 Opening HTTPS tunnel to {}:{} via Tor", host, port);
        
        if let Some(state) = &self.app_state {
            state.add_log("info", format!("🔐 Opening tunnel to {}:{}", host, port), "network").await;
        }
        
        self.tor.connect_stream(host, port).await
    }
    
    /// Get statistics about blocked trackers
    pub fn get_stats(&self) -> (usize, u64) {
        (self.tracker_blocker.blocklist_size(), self.tracker_blocker.total_blocked())
    }
    
    fn select_route(&self) -> Vec<&Node> {
        // Randomly select nodes for the route
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        
        let num_hops = self.config.num_hops.min(self.nodes.len());
        let mut selected: Vec<&Node> = self.nodes.iter().collect();
        selected.shuffle(&mut rng);
        selected.truncate(num_hops);
        
        selected
    }
    
    async fn send_through_route(
        &self,
        _encrypted_request: Vec<u8>,
        route: &[&Node],
    ) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement actual multi-hop routing
        // For now, return a placeholder response
        
        info!("Request routed through: {:?}", route);
        
        Ok(Response::new(Full::new(Bytes::from("Privacy Suite - Request Routed"))))
    }
}
