use axum::{
    extract::State,
    response::sse::{Event, Sse},
    routing::{get, post, put},
    Json, Router,
};
use futures::stream::{Stream, self};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, error};
use reqwest::Proxy as ReqwestProxy;
use crate::kill_switch::KillSwitch;
use crate::proxy::ProxyServer;
use crate::config::Config;
use crate::system_proxy::{self as sys_proxy, SystemProxy};
use crate::system_proxy;
use crate::dns::DnsResolver;
use crate::fingerprint::BrowserFingerprint;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stats {
    pub tor_connected: bool,
    pub kill_switch_active: bool,
    pub requests_blocked: u64,
    pub trackers_blocked: u64,
    pub webrtc_blocked: u64,
    pub ipv6_blocked: u64,
    pub fingerprints_randomized: u64,
    pub last_fingerprint_at: Option<u64>,
    pub last_fingerprint_user_agent: Option<String>,
    pub total_requests: u64,
    pub proxy_running: bool,
    pub auto_proxy_enabled: bool,
    pub uptime_seconds: u64,
    pub security_threats_detected: u64,
    pub exit_country: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub category: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<LogDetails>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogDetails {
    pub url: Option<String>,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub port: Option<u16>,
    pub method: Option<String>,
    pub client_ip: Option<String>,
    pub threat_type: Option<String>,
    pub reason: Option<String>,
    pub request_headers: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize)]
struct SelfTestCheck {
    name: String,
    ok: bool,
    detail: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
struct SelfTestResult {
    ok: bool,
    checks: Vec<SelfTestCheck>,
}

#[derive(Clone)]
pub struct ApiState {
    pub stats: Arc<RwLock<Stats>>,
    pub logs: Arc<RwLock<Vec<LogEntry>>>,
    pub start_time: std::time::Instant,
    pub connected_time: Arc<RwLock<Option<std::time::Instant>>>,
    pub total_connected_duration: Arc<RwLock<u64>>,
    pub kill_switch: Option<KillSwitch>,
    pub config: Arc<Config>,
    pub proxy_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    pub system_proxy: Arc<RwLock<SystemProxy>>,
}

async fn wait_for_proxy_ready() -> bool {
    let proxy = match ReqwestProxy::all("http://127.0.0.1:8888") {
        Ok(p) => p,
        Err(_) => return false,
    };
    let client = match reqwest::Client::builder()
        .proxy(proxy)
        .timeout(Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };

    for attempt in 1..=10 {
        let res = client
            .get("https://check.torproject.org/api/ip")
            .send()
            .await;
        if let Ok(resp) = res {
            if resp.status().is_success() {
                return true;
            }
        }
        tokio::time::sleep(Duration::from_millis(500 * attempt)).await;
    }

    false
}

impl ApiState {
    // Create API state
    pub fn new(config: Config) -> Self {
        Self {
            stats: Arc::new(RwLock::new(Stats {
                tor_connected: false,
                kill_switch_active: false,
                requests_blocked: 0,
                trackers_blocked: 0,
                webrtc_blocked: 0,
                ipv6_blocked: 0,
                fingerprints_randomized: 0,
                last_fingerprint_at: None,
                last_fingerprint_user_agent: None,
                total_requests: 0,
                proxy_running: false,
                auto_proxy_enabled: false,
                uptime_seconds: 0,
                security_threats_detected: 0,
                exit_country: None,
            })),
            logs: Arc::new(RwLock::new(Vec::new())),
            start_time: std::time::Instant::now(),
            connected_time: Arc::new(RwLock::new(None)),
            total_connected_duration: Arc::new(RwLock::new(0)),
            kill_switch: None,
            config: Arc::new(config),
            proxy_handle: Arc::new(RwLock::new(None)),
            system_proxy: Arc::new(RwLock::new(SystemProxy::new())),
        }
    }
    
    pub fn with_kill_switch(mut self, kill_switch: KillSwitch) -> Self {
        self.kill_switch = Some(kill_switch);
        self
    }
    
    pub fn with_system_proxy(mut self, system_proxy: Arc<RwLock<SystemProxy>>) -> Self {
        self.system_proxy = system_proxy;
        self
    }

    // Add a log entry
    pub async fn add_log(&self, level: &str, message: String, category: &str) {
        self.add_log_with_details(level, message, category, None).await;
    }

    // Add a log entry with details
    pub async fn add_log_with_details(&self, level: &str, message: String, category: &str, details: Option<LogDetails>) {
        let mut logs = self.logs.write().await;
        logs.push(LogEntry {
            timestamp: chrono::Local::now().format("%H:%M:%S%.3f").to_string(),
            level: level.to_string(),
            message,
            category: category.to_string(),
            details,
        });
        if logs.len() > 2000 {
            logs.remove(0);
        }
    }

    // Update stats with a closure
    pub async fn update_stats<F>(&self, f: F)
    where
        F: FnOnce(&mut Stats),
    {
        let mut stats = self.stats.write().await;
        f(&mut *stats);
        
        if let Some(connected_since) = *self.connected_time.read().await {
            stats.uptime_seconds = connected_since.elapsed().as_secs();
        } else {
            stats.uptime_seconds = 0;
        }
    }
}

// GET /api/stats
async fn get_stats(State(state): State<ApiState>) -> Json<Stats> {
    let mut stats = state.stats.read().await.clone();
    
    if let Some(connected_since) = *state.connected_time.read().await {
        stats.uptime_seconds = connected_since.elapsed().as_secs();
    } else {
        stats.uptime_seconds = 0;
    }
    
    Json(stats)
}

// GET /api/logs
async fn get_logs(State(state): State<ApiState>) -> Json<Vec<LogEntry>> {
    let logs = state.logs.read().await.clone();
    Json(logs)
}

#[derive(Deserialize)]
struct LogFilter {
    category: Option<String>,
    level: Option<String>,
}

async fn get_filtered_logs(
    State(state): State<ApiState>,
    Json(filter): Json<LogFilter>,
) -> Json<Vec<LogEntry>> {
    let logs = state.logs.read().await;
    let filtered: Vec<LogEntry> = logs
        .iter()
        .filter(|log| {
            let category_match = filter
                .category
                .as_ref()
                .map(|c| &log.category == c)
                .unwrap_or(true);
            let level_match = filter
                .level
                .as_ref()
                .map(|l| &log.level == l)
                .unwrap_or(true);
            category_match && level_match
        })
        .cloned()
        .collect();
    Json(filtered)
}

async fn stats_stream(
    State(state): State<ApiState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream::unfold(state, |state| async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let mut stats = state.stats.read().await.clone();
        if let Some(connected_since) = *state.connected_time.read().await {
            stats.uptime_seconds = connected_since.elapsed().as_secs();
        } else {
            stats.uptime_seconds = 0;
        }
        let event = Event::default().json_data(stats).ok()?;
        Some((Ok(event), state))
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}

async fn logs_stream(
    State(state): State<ApiState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream::unfold(state, |state| async move {
        tokio::time::sleep(Duration::from_millis(200)).await;
        let logs = state.logs.read().await.clone();
        let event = Event::default().json_data(logs).ok()?;
        Some((Ok(event), state))
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}

#[derive(Deserialize)]
struct KillSwitchToggle {
    enabled: bool,
}

// PUT /api/killswitch
async fn toggle_kill_switch(
    State(state): State<ApiState>,
    Json(toggle): Json<KillSwitchToggle>,
) -> Json<Stats> {
    if let Some(ref kill_switch) = state.kill_switch {
        kill_switch.set_enabled(toggle.enabled).await;
        state.update_stats(|s| s.kill_switch_active = toggle.enabled).await;
        
        let log_msg = if toggle.enabled {
            "🔒 Kill switch ENABLED - Will block traffic if Tor disconnects".to_string()
        } else {
            "⚠️ Kill switch DISABLED - Traffic may leak if Tor fails!".to_string()
        };
        state.add_log("info", log_msg, "general").await;
    }
    
    let mut stats = state.stats.read().await.clone();
    if let Some(connected_since) = *state.connected_time.read().await {
        stats.uptime_seconds = connected_since.elapsed().as_secs();
    } else {
        stats.uptime_seconds = 0;
    }
    Json(stats)
}

// POST /api/shutdown
async fn shutdown(
    State(state): State<ApiState>,
) -> Json<bool> {
    state.add_log("info", "Shutdown requested from GUI".to_string(), "general").await;
    
    if let Some(ref ks) = state.kill_switch {
        ks.set_enabled(false).await;
    }
    if sys_proxy::is_elevated() {
        let _ = state.system_proxy.write().await.disable();
    }
    if let Some(handle) = state.proxy_handle.write().await.take() {
        handle.abort();
    }
    std::process::exit(0);
}

#[derive(Deserialize)]
struct ExitCountryChange {
    country: Option<String>,
}

// POST /api/exit-country
async fn change_exit_country(
    State(state): State<ApiState>,
    Json(change): Json<ExitCountryChange>,
) -> Json<Stats> {
    let country_name = if let Some(ref country) = change.country {
        match country.as_str() {
            "us" => "United States 🇺🇸",
            "uk" => "United Kingdom 🇬🇧",
            "de" => "Germany 🇩🇪",
            "nl" => "Netherlands 🇳🇱",
            "fr" => "France 🇫🇷",
            "se" => "Sweden 🇸🇪",
            "ch" => "Switzerland 🇨🇭",
            "ca" => "Canada 🇨🇦",
            "au" => "Australia 🇦🇺",
            "jp" => "Japan 🇯🇵",
            _ => country.as_str(),
        }
    } else {
        "Auto (Random)"
    };
    state.update_stats(|s| s.exit_country = change.country.clone()).await;
    state.add_log("info", format!("🌍 Exit location changed to: {}", country_name), "network").await;
    let mut stats = state.stats.read().await.clone();
    if let Some(connected_since) = *state.connected_time.read().await {
        stats.uptime_seconds = connected_since.elapsed().as_secs();
    } else {
        stats.uptime_seconds = 0;
    }
    Json(stats)
}

#[derive(Deserialize)]
struct ConnectionToggle {
    connect: bool,
    exit_country: Option<String>,
}

// POST /api/connection
async fn toggle_connection(
    State(state): State<ApiState>,
    Json(toggle): Json<ConnectionToggle>,
) -> Json<Stats> {
    if toggle.connect {
        let is_already_running = state.stats.read().await.proxy_running;
        let has_handle = state.proxy_handle.read().await.is_some();
        
        if is_already_running || has_handle {
            state.add_log("warn", "Already connected or connecting...".to_string(), "general").await;
            let mut stats = state.stats.read().await.clone();
            if let Some(connected_since) = *state.connected_time.read().await {
                stats.uptime_seconds = connected_since.elapsed().as_secs();
            } else {
                stats.uptime_seconds = 0;
            }
            return Json(stats);
        }
        state.add_log("info", "🔌 Connecting to Privacy Suite...".to_string(), "general").await;
        state.add_log("info", "🔐 Establishing encrypted Tor connection...".to_string(), "general").await;
        if let Some(ref country) = toggle.exit_country {
            let country_name = match country.as_str() {
                "us" => "United States 🇺🇸",
                "uk" => "United Kingdom 🇬🇧",
                "de" => "Germany 🇩🇪",
                "nl" => "Netherlands 🇳🇱",
                "fr" => "France 🇫🇷",
                "se" => "Sweden 🇸🇪",
                "ch" => "Switzerland 🇨🇭",
                "ca" => "Canada 🇨🇦",
                "au" => "Australia 🇦🇺",
                "jp" => "Japan 🇯🇵",
                _ => country.as_str(),
            };
            state.add_log("info", format!("🌍 Exit location set to: {}", country_name), "network").await;
            state.update_stats(|s| s.exit_country = Some(country.clone())).await;
        } else {
            state.add_log("info", "🌍 Exit location: Auto (Random)".to_string(), "network").await;
            state.update_stats(|s| s.exit_country = None).await;
        }
        
        // Configure firewall first (doesn't break internet if Tor fails)
        let is_elevated = sys_proxy::is_elevated();
        if is_elevated {
            match sys_proxy::allow_inbound_port(8888) {
                Ok(_) => {
                    state.add_log("info", "✅ Firewall opened for LAN devices (TCP 8888)".to_string(), "general").await;
                }
                Err(e) => {
                    state.add_log("warn", format!("Failed to open firewall for LAN: {}", e), "general").await;
                }
            }
        }
        
        let proxy_state = state.clone();
        let config = (*state.config).clone();
        
        let handle = tokio::spawn(async move {
            match ProxyServer::new_with_listener(config.clone(), Some(proxy_state.clone())).await {
                Ok((proxy, listener)) => {
                    proxy_state.add_log("info", "✅ Connected to Tor! Using 6,000+ volunteer nodes".into(), "general").await;
                    proxy_state.add_log("info", "🌐 Proxy bound to 0.0.0.0:8888".into(), "network").await;
                    
                    // Start accept loop in background FIRST, before enabling system proxy
                    let listen_state = proxy_state.clone();
                    let accept_handle = tokio::spawn(async move {
                        if let Err(e) = proxy.run(listener).await {
                            error!("Proxy accept loop error: {}", e);
                            listen_state.add_log("error", format!("Proxy error: {}", e), "general").await;
                        }
                    });
                    
                    // Give the accept loop a moment to start
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    
                    proxy_state.add_log("info", "📱 Other devices can connect using your LAN IP:8888".into(), "network").await;
                    
                    // Mark proxy running, but don't claim Tor connectivity until verified
                    proxy_state.update_stats(|s| {
                        s.proxy_running = true;
                        s.tor_connected = false;
                        s.requests_blocked = 0;
                        s.trackers_blocked = 0;
                        s.webrtc_blocked = 0;
                        s.ipv6_blocked = 0;
                        s.fingerprints_randomized = 0;
                        s.total_requests = 0;
                        s.uptime_seconds = 0;
                        s.security_threats_detected = 0;
                    }).await;

                    // Continuously verify proxy connectivity, then enable system proxy
                    let readiness_state = proxy_state.clone();
                    tokio::spawn(async move {
                        let mut logged_wait = false;
                        loop {
                            if !logged_wait {
                                readiness_state.add_log("info", "⏳ Verifying proxy connectivity before enabling system proxy...".to_string(), "general").await;
                                logged_wait = true;
                            }
                            if wait_for_proxy_ready().await {
                                readiness_state.add_log("info", "✅ Proxy connectivity verified".to_string(), "general").await;
                                if is_elevated {
                                    match readiness_state.system_proxy.write().await.enable("127.0.0.1:8888") {
                                        Ok(_) => {
                                            readiness_state.add_log("info", "✅ System proxy configured - all apps will be protected".to_string(), "general").await;
                                            readiness_state.update_stats(|s| s.auto_proxy_enabled = true).await;
                                        }
                                        Err(e) => {
                                            readiness_state.add_log("warn", format!("Failed to configure system proxy: {}", e), "general").await;
                                        }
                                    }
                                }
                                readiness_state.update_stats(|s| s.tor_connected = true).await;

                                // Start a watchdog to disable system proxy if Tor/proxy stops working
                                let watchdog_state = readiness_state.clone();
                                tokio::spawn(async move {
                                    let mut failures = 0u32;
                                    loop {
                                        if wait_for_proxy_ready().await {
                                            failures = 0;
                                        } else {
                                            failures += 1;
                                        }

                                        if failures >= 3 {
                                            watchdog_state.add_log("error", "❌ Proxy connectivity lost. Disabling system proxy to restore internet.".to_string(), "general").await;
                                            if let Err(e) = watchdog_state.system_proxy.write().await.disable() {
                                                error!("Failed to disable system proxy after connectivity loss: {}", e);
                                            } else {
                                                watchdog_state.update_stats(|s| {
                                                    s.auto_proxy_enabled = false;
                                                    s.tor_connected = false;
                                                }).await;
                                            }
                                            break;
                                        }

                                        tokio::time::sleep(Duration::from_secs(10)).await;
                                    }
                                });
                                break;
                            }
                            tokio::time::sleep(Duration::from_secs(5)).await;
                        }
                    });
                    
                    *proxy_state.connected_time.write().await = Some(std::time::Instant::now());
                    *proxy_state.total_connected_duration.write().await = 0;
                    
                    info!("✅ Privacy Suite proxy is running!");
                    proxy_state.add_log("info", "✅ All systems operational - Privacy Suite is LIVE".to_string(), "general").await;
                    
                    // Wait for accept loop to finish (it runs until disconnect)
                    let _ = accept_handle.await;
                    
                    // Clean up when proxy stops
                    if let Some(connected_since) = proxy_state.connected_time.write().await.take() {
                        let session_duration = connected_since.elapsed().as_secs();
                        *proxy_state.total_connected_duration.write().await += session_duration;
                    }
                    
                    proxy_state.update_stats(|s| {
                        s.proxy_running = false;
                        s.tor_connected = false;
                    }).await;
                    
                    proxy_state.add_log("info", "Proxy stopped".to_string(), "general").await;
                }
                Err(e) => {
                    error!("Failed to connect to Tor: {}", e);
                    proxy_state.add_log("error", format!("❌ Failed to connect to Tor: {}", e), "general").await;
                    proxy_state.add_log("warn", "💡 Try: 1) Check internet connection 2) Disable firewall temporarily 3) Use bridges if Tor is blocked".to_string(), "general").await;
                    
                    // Disable system proxy if Tor failed to prevent breaking internet
                    if is_elevated {
                        if let Err(e) = proxy_state.system_proxy.write().await.disable() {
                            error!("Failed to restore proxy after Tor failure: {}", e);
                        } else {
                            proxy_state.add_log("info", "System proxy disabled (Tor connection failed)".to_string(), "general").await;
                        }
                    }
                    
                    proxy_state.update_stats(|s| {
                        s.proxy_running = false;
                        s.tor_connected = false;
                        s.auto_proxy_enabled = false;
                    }).await;
                }
            }
        });
        
        *state.proxy_handle.write().await = Some(handle);
        
        state.add_log("info", "Connection initiated...".to_string(), "general").await;
    } else {
        state.add_log("info", "🔌 Disconnecting from Privacy Suite...".to_string(), "general").await;
        if sys_proxy::is_elevated() {
            match state.system_proxy.write().await.disable() {
                Ok(_) => {
                    state.add_log("info", "System proxy disabled".to_string(), "general").await;
                    state.update_stats(|s| s.auto_proxy_enabled = false).await;
                }
                Err(e) => {
                    state.add_log("warn", format!("Failed to disable system proxy: {}", e), "general").await;
                }
            }
        }
        
        if let Some(handle) = state.proxy_handle.write().await.take() {
            *state.connected_time.write().await = None;
            *state.total_connected_duration.write().await = 0;
            
            handle.abort();
            state.update_stats(|s| {
                s.proxy_running = false;
                s.tor_connected = false;
                s.uptime_seconds = 0;
            }).await;
            state.add_log("info", "✅ Disconnected successfully".to_string(), "general").await;
        } else {
            state.add_log("warn", "No active connection to disconnect".to_string(), "general").await;
        }
    }
    
    let mut stats = state.stats.read().await.clone();
    stats.uptime_seconds = state.start_time.elapsed().as_secs();
    Json(stats)
}

// POST /api/self-test
async fn self_test(State(state): State<ApiState>) -> Json<SelfTestResult> {
    let mut checks: Vec<SelfTestCheck> = Vec::new();

    let stats = state.stats.read().await.clone();
    checks.push(SelfTestCheck {
        name: "Proxy running".to_string(),
        ok: stats.proxy_running,
        detail: None,
    });
    checks.push(SelfTestCheck {
        name: "Tor connected".to_string(),
        ok: stats.tor_connected,
        detail: None,
    });
    checks.push(SelfTestCheck {
        name: "Kill switch".to_string(),
        ok: stats.kill_switch_active,
        detail: None,
    });

    let admin = sys_proxy::is_elevated();
    checks.push(SelfTestCheck {
        name: "Admin privileges".to_string(),
        ok: admin,
        detail: if admin { None } else { Some("Limited system proxy control".to_string()) },
    });

    let fingerprint_enabled = state.config.fingerprint_protection;
    checks.push(SelfTestCheck {
        name: "Fingerprint protection".to_string(),
        ok: fingerprint_enabled,
        detail: None,
    });

    let fp = BrowserFingerprint::random();
    checks.push(SelfTestCheck {
        name: "Fingerprint generation".to_string(),
        ok: !fp.user_agent.is_empty(),
        detail: None,
    });

    let dns_check = tokio::time::timeout(Duration::from_secs(3), async {
        let resolver = DnsResolver::new().await.map_err(|e| e.to_string())?;
        resolver.resolve("example.com").await.map_err(|e| e.to_string())?;
        Ok::<(), String>(())
    }).await;

    let dns_ok = matches!(dns_check, Ok(Ok(())));
    checks.push(SelfTestCheck {
        name: "DNS resolution".to_string(),
        ok: dns_ok,
        detail: if dns_ok { None } else { Some("Lookup failed".to_string()) },
    });

    let ok = checks.iter().all(|c| c.ok);
    let summary = if ok { "Self-test passed" } else { "Self-test had failures" };
    state.add_log("info", summary.to_string(), "general").await;
    for check in &checks {
        let level = if check.ok { "info" } else { "warn" };
        let detail = check.detail.clone().unwrap_or_else(|| "OK".to_string());
        state
            .add_log(level, format!("Self-test: {} - {}", check.name, detail), "general")
            .await;
    }

    Json(SelfTestResult { ok, checks })
}

// Start HTTP API server
pub async fn start_web_api(
    state: ApiState,
    port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/stats", get(get_stats))
        .route("/api/stats/stream", get(stats_stream))
        .route("/api/logs", get(get_logs))
        .route("/api/logs/filter", post(get_filtered_logs))
        .route("/api/logs/stream", get(logs_stream))
        .route("/api/killswitch", put(toggle_kill_switch))
        .route("/api/connection", post(toggle_connection))
        .route("/api/exit-country", put(change_exit_country))
        .route("/api/shutdown", post(shutdown))
        .route("/api/self-test", post(self_test))
        .layer(cors)
        .with_state(state);

    let addr = format!("127.0.0.1:{}", port);
    info!("🌐 Web API listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
