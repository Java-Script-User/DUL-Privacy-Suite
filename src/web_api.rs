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
use tracing::info;
use crate::kill_switch::KillSwitch;
use crate::proxy::ProxyServer;
use crate::config::Config;
use crate::system_proxy::{self as sys_proxy, SystemProxy};
use crate::system_proxy;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stats {
    pub tor_connected: bool,
    pub kill_switch_active: bool,
    pub requests_blocked: u64,
    pub trackers_blocked: u64,
    pub webrtc_blocked: u64,
    pub ipv6_blocked: u64,
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
    pub category: String, // "tracker", "webrtc", "ipv6", "general", "network", "security"
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

impl ApiState {
    pub fn new(config: Config) -> Self {
        Self {
            stats: Arc::new(RwLock::new(Stats {
                tor_connected: false,
                kill_switch_active: false,
                requests_blocked: 0,
                trackers_blocked: 0,
                webrtc_blocked: 0,
                ipv6_blocked: 0,
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

    pub async fn add_log(&self, level: &str, message: String, category: &str) {
        self.add_log_with_details(level, message, category, None).await;
    }

    pub async fn add_log_with_details(&self, level: &str, message: String, category: &str, details: Option<LogDetails>) {
        let mut logs = self.logs.write().await;
        logs.push(LogEntry {
            timestamp: chrono::Local::now().format("%H:%M:%S%.3f").to_string(),
            level: level.to_string(),
            message,
            category: category.to_string(),
            details,
        });
        // Keep only last 2000 logs for detailed tracking
        if logs.len() > 2000 {
            logs.remove(0);
        }
    }

    pub async fn update_stats<F>(&self, f: F)
    where
        F: FnOnce(&mut Stats),
    {
        let mut stats = self.stats.write().await;
        f(&mut *stats);
        
        // Calculate only connected session duration
        if let Some(connected_since) = *self.connected_time.read().await {
            stats.uptime_seconds = connected_since.elapsed().as_secs();
        } else {
            stats.uptime_seconds = 0;
        }
    }
}

async fn get_stats(State(state): State<ApiState>) -> Json<Stats> {
    let mut stats = state.stats.read().await.clone();
    
    // Calculate only connected session duration (not total app uptime)
    if let Some(connected_since) = *state.connected_time.read().await {
        stats.uptime_seconds = connected_since.elapsed().as_secs();
    } else {
        stats.uptime_seconds = 0;
    }
    
    Json(stats)
}

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
        // Calculate only connected session duration
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
    // Calculate only connected session duration
    if let Some(connected_since) = *state.connected_time.read().await {
        stats.uptime_seconds = connected_since.elapsed().as_secs();
    } else {
        stats.uptime_seconds = 0;
    }
    Json(stats)
}

async fn shutdown(
    State(state): State<ApiState>,
) -> Json<bool> {
    state.add_log("info", "Shutdown requested from GUI".to_string(), "general").await;
    
    // Disable kill switch
    if let Some(ref ks) = state.kill_switch {
        ks.set_enabled(false).await;
    }
    
    // Disable system proxy
    if sys_proxy::is_elevated() {
        let _ = state.system_proxy.write().await.disable();
    }
    
    // Stop proxy
    if let Some(handle) = state.proxy_handle.write().await.take() {
        handle.abort();
    }
    
    // Exit the process
    std::process::exit(0);
}

#[derive(Deserialize)]
struct ExitCountryChange {
    country: Option<String>,
}

async fn change_exit_country(
    State(state): State<ApiState>,
    Json(change): Json<ExitCountryChange>,
) -> Json<Stats> {
    // Update the exit country preference
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
    
    // Tor circuit restart would be implemented here
    // For now, we just update the preference for the next connection
    
    let mut stats = state.stats.read().await.clone();
    // Calculate only connected session duration
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

async fn toggle_connection(
    State(state): State<ApiState>,
    Json(toggle): Json<ConnectionToggle>,
) -> Json<Stats> {
    if toggle.connect {
        // Check if already connecting/connected
        let is_already_running = state.stats.read().await.proxy_running;
        let has_handle = state.proxy_handle.read().await.is_some();
        
        if is_already_running || has_handle {
            state.add_log("warn", "Already connected or connecting...".to_string(), "general").await;
            let mut stats = state.stats.read().await.clone();
            // Calculate only connected session duration
            if let Some(connected_since) = *state.connected_time.read().await {
                stats.uptime_seconds = connected_since.elapsed().as_secs();
            } else {
                stats.uptime_seconds = 0;
            }
            return Json(stats);
        }
        
        // Start connection
        state.add_log("info", "🔌 Connecting to Privacy Suite...".to_string(), "general").await;
        state.add_log("info", "🔐 Establishing encrypted Tor connection...".to_string(), "general").await;
        
        // Log exit country selection
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
        
        // Configure system proxy if running as admin
        if sys_proxy::is_elevated() {
            let proxy_addr = (*state.config).proxy_addr();
            match state.system_proxy.write().await.enable(&proxy_addr) {
                Ok(_) => {
                    state.add_log("info", "✅ System proxy configured - all apps will be protected".to_string(), "general").await;
                    state.update_stats(|s| s.auto_proxy_enabled = true).await;
                }
                Err(e) => {
                    state.add_log("warn", format!("Failed to configure system proxy: {}", e), "general").await;
                }
            }
        }
        
        let proxy_state = state.clone();
        let config = (*state.config).clone();
        
        let handle = tokio::spawn(async move {
            match ProxyServer::new(config.clone(), Some(proxy_state.clone())).await {
                Ok(proxy) => {
                    proxy_state.add_log("info", "✅ Connected to Tor! Using 6,000+ volunteer nodes".into(), "general").await;
                    proxy_state.add_log("info", "🌐 Proxy listening on all network interfaces (0.0.0.0:8888)".into(), "network").await;
                    proxy_state.add_log("info", "📱 Other devices can connect using your LAN IP:8888".into(), "network").await;
                    
                    // Reset counters for new session
                    proxy_state.update_stats(|s| {
                        s.proxy_running = true;
                        s.tor_connected = true;
                        s.requests_blocked = 0;
                        s.trackers_blocked = 0;
                        s.webrtc_blocked = 0;
                        s.ipv6_blocked = 0;
                        s.total_requests = 0;
                        s.uptime_seconds = 0;
                        s.security_threats_detected = 0;
                    }).await;
                    
                    // Start tracking connected time for this session
                    *proxy_state.connected_time.write().await = Some(std::time::Instant::now());
                    *proxy_state.total_connected_duration.write().await = 0;
                    
                    info!("✅ Privacy Suite proxy is running!");
                    proxy_state.add_log("info", "✅ All systems operational - Privacy Suite is LIVE".to_string(), "general").await;
                    
                    let _ = proxy.run().await;
                    
                    // Stop tracking connected time and add to total
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
                    proxy_state.add_log("error", format!("Failed to start proxy: {}", e), "general").await;
                }
            }
        });
        
        *state.proxy_handle.write().await = Some(handle);
        
        state.add_log("info", "Connection initiated...".to_string(), "general").await;
    } else {
        // Stop connection
        state.add_log("info", "🔌 Disconnecting from Privacy Suite...".to_string(), "general").await;
        
        // Disable system proxy if it was enabled
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
            // Clear connected duration
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
        .layer(cors)
        .with_state(state);

    let addr = format!("127.0.0.1:{}", port);
    info!("🌐 Web API listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
