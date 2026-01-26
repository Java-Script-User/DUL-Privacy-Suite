use tracing::{info, warn, error};
use tracing_subscriber;

mod config;
mod crypto;
mod dns;
mod fingerprint;
mod network;
mod blockchain;
mod proxy;
mod routing;
mod tor_network;
mod blocklist;
mod webrtc_protection;
mod kill_switch;
mod ipv6_protection;
mod web_api;
mod system_proxy;

use config::Config;
use web_api::ApiState;
fn get_lan_ip() -> Option<String> {
    use std::net::UdpSocket;
    // Detect local interface
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Logging
    tracing_subscriber::fmt()
        .with_env_filter("privacy_suite=info")
        .init();

    info!("🚀 Starting Privacy Suite...");
    
    // Config
    let config = Config::load_or_create()?;
    info!("Configuration loaded from: {}", config.config_path().display());
    // Core services
    let kill_switch = kill_switch::KillSwitch::new();
    let sys_proxy = std::sync::Arc::new(tokio::sync::RwLock::new(system_proxy::SystemProxy::new()));
    let api_state = ApiState::new(config.clone())
        .with_kill_switch(kill_switch.clone())
        .with_system_proxy(sys_proxy.clone());
    api_state.add_log("info", "Privacy Suite starting...".to_string(), "general").await;
    api_state.add_log("info", "ℹ️ Click CONNECT button to start privacy protection".to_string(), "general").await;
    // Environment
    let is_admin = system_proxy::is_elevated();
    let lan_ip = get_lan_ip();
    info!("Admin status: {}", is_admin);
    
    if let Some(ref ip) = lan_ip {
        info!("🌐 LAN IP Address: {}", ip);
        info!("📱 Other devices can use: {}:8888", ip);
    }
    
    if is_admin {
        info!("Running with administrator privileges");
        api_state.add_log("info", "✅ Running with administrator privileges - system-wide protection available".to_string(), "general").await;
    } else {
        info!("💡 Tip: Run as Administrator for automatic system-wide proxy");
        if let Some(ref ip) = lan_ip {
            info!("Or manually configure devices to use: {}:8888", ip);
            api_state.add_log("warn", format!("⚠️ Not running as administrator - manually configure devices to use: {}:8888", ip), "general").await;
        } else {
            info!("Or manually configure your browser to use: {}", config.proxy_addr());
            api_state.add_log("warn", "⚠️ Not running as administrator - manual browser setup required".to_string(), "general").await;
        }
    }
    
    // API server
    info!("🌐 Starting Web API on http://127.0.0.1:3030");
    info!("🌐 Starting Web API on http://127.0.0.1:3030");
    let web_api_state = api_state.clone();
    tokio::spawn(async move {
        if let Err(e) = web_api::start_web_api(web_api_state, 3030).await {
            eprintln!("Web API error: {}", e);
        }
    });
    
    // Give API time to boot
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    info!("✅ Privacy Suite ready!");
    info!("📊 Web GUI: http://127.0.0.1:1420");
    info!("🔌 Proxy: {} (disconnected - click Connect in GUI)", config.proxy_addr());

    if is_admin {
        api_state.add_log("info", "Auto-starting protection and system proxy...".to_string(), "general").await;
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build();
        if let Ok(client) = client {
            for attempt in 1..=10 {
                let result = client
                    .post("http://127.0.0.1:3030/api/connection")
                    .json(&serde_json::json!({ "connect": true }))
                    .send()
                    .await;
                if result.is_ok() {
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                if attempt == 10 {
                    api_state.add_log("warn", "Auto-start failed. Use CONNECT button.".to_string(), "general").await;
                }
            }
        }
    } else {
        api_state.add_log("warn", "Run as Administrator for automatic system proxy (seamless mode)".to_string(), "general").await;
    }
    
    if let Some(ref ip) = lan_ip {
        info!("🌐 Network-wide access: Configure devices to use {}:8888", ip);
        api_state.add_log("info", format!("🌐 Network-wide proxy available at: {}:8888", ip), "general").await;
    }
    
    info!("Press Ctrl+C to stop");
    
    // Wait for shutdown
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");
    api_state.add_log("info", "Shutting down Privacy Suite...".to_string(), "general").await;
    // Safe shutdown
    if let Some(ref ks) = api_state.kill_switch {
        info!("Disabling kill switch...");
        ks.set_enabled(false).await;
        api_state.add_log("info", "Kill switch disabled".to_string(), "general").await;
    }
    if system_proxy::is_elevated() {
        info!("Restoring original proxy settings...");
        let proxy = sys_proxy.read().await;
        if let Err(e) = proxy.disable() {
            error!("Failed to restore proxy: {}", e);
        } else {
            api_state.add_log("info", "Proxy settings restored".to_string(), "general").await;
        }
    }
    
    info!("✅ Shutdown complete");
    
    Ok(())
}
