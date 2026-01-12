use serde::{Deserialize, Serialize};
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize)]
struct Stats {
    tor_connected: bool,
    kill_switch_active: bool,
    requests_blocked: u64,
    trackers_blocked: u64,
    webrtc_blocked: u64,
    ipv6_blocked: u64,
    total_requests: u64,
    proxy_running: bool,
    auto_proxy_enabled: bool,
    uptime_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    category: String,
    message: String,
}

#[tauri::command]
async fn get_stats() -> Result<Stats, String> {
    println!("get_stats: Starting request to backend...");
    
    let response = reqwest::get("http://127.0.0.1:3030/api/stats")
        .await
        .map_err(|e| {
            println!("get_stats: Request failed: {}", e);
            format!("Request failed: {}", e)
        })?;
    
    println!("get_stats: Response status: {}", response.status());
    
    let text = response.text().await
        .map_err(|e| {
            println!("get_stats: Failed to read response text: {}", e);
            format!("Failed to read response: {}", e)
        })?;
    
    println!("get_stats: Response body: {}", text);
    
    let stats: Stats = serde_json::from_str(&text)
        .map_err(|e| {
            println!("get_stats: JSON parse failed: {}", e);
            format!("JSON parse error: {} - Body: {}", e, text)
        })?;
    
    println!("get_stats: Success!");
    Ok(stats)
}

#[tauri::command]
async fn get_logs() -> Result<Vec<LogEntry>, String> {
    let response = reqwest::get("http://127.0.0.1:3030/api/logs")
        .await
        .map_err(|e| format!("Failed to fetch logs: {}", e))?;
    
    let logs = response
        .json::<Vec<LogEntry>>()
        .await
        .map_err(|e| format!("Failed to parse logs: {}", e))?;
    
    Ok(logs)
}

#[derive(Debug, Serialize, Deserialize)]
struct KillSwitchToggle {
    enabled: bool,
}

#[tauri::command]
async fn toggle_kill_switch(enabled: bool) -> Result<Stats, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    let response = client
        .put("http://127.0.0.1:3030/api/killswitch")
        .json(&KillSwitchToggle { enabled })
        .send()
        .await
        .map_err(|e| format!("Failed to toggle kill switch: {}", e))?;
    
    let stats = response
        .json::<Stats>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    Ok(stats)
}

#[derive(Debug, Serialize, Deserialize)]
struct ConnectionToggle {
    connect: bool,
}

#[tauri::command]
async fn toggle_connection(connect: bool) -> Result<Stats, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    let response = client
        .post("http://127.0.0.1:3030/api/connection")
        .json(&ConnectionToggle { connect })
        .send()
        .await
        .map_err(|e| format!("Failed to toggle connection: {}", e))?;
    
    let stats = response
        .json::<Stats>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    Ok(stats)
}

#[tauri::command]
async fn shutdown_backend() -> Result<(), String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    // Try to shutdown backend gracefully
    let _ = client
        .post("http://127.0.0.1:3030/api/shutdown")
        .send()
        .await;
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![get_stats, get_logs, toggle_kill_switch, toggle_connection, shutdown_backend])
        .setup(|app| {
            // Auto-start backend if not already running
            std::thread::spawn(move || {
                // Check if backend is already running
                let backend_running = std::net::TcpStream::connect("127.0.0.1:3030").is_ok();
                
                if !backend_running {
                    println!("Backend not running, starting it...");
                    
                    // Get the directory where the GUI executable is located
                    let exe_path = std::env::current_exe().unwrap_or_default();
                    let exe_dir = exe_path.parent().unwrap_or(std::path::Path::new("."));
                    let backend_path = exe_dir.join("privacy_suite.exe");
                    
                    println!("Looking for backend at: {:?}", backend_path);
                    
                    if backend_path.exists() {
                        // Start the backend process with admin rights
                        #[cfg(target_os = "windows")]
                        {
                            use std::os::windows::process::CommandExt;
                            const CREATE_NO_WINDOW: u32 = 0x08000000;
                            
                            let result = std::process::Command::new("powershell")
                                .args(&[
                                    "-Command",
                                    &format!("Start-Process -FilePath '{}' -Verb RunAs -WindowStyle Hidden", backend_path.display())
                                ])
                                .creation_flags(CREATE_NO_WINDOW)
                                .spawn();
                            
                            match result {
                                Ok(_) => {
                                    println!("Backend started successfully");
                                    // Wait for backend to initialize
                                    for i in 0..30 {
                                        std::thread::sleep(std::time::Duration::from_millis(500));
                                        if std::net::TcpStream::connect("127.0.0.1:3030").is_ok() {
                                            println!("Backend is ready after {} attempts", i + 1);
                                            break;
                                        }
                                    }
                                }
                                Err(e) => println!("Failed to start backend: {}", e),
                            }
                        }
                    } else {
                        println!("Backend executable not found at {:?}", backend_path);
                    }
                } else {
                    println!("Backend already running on port 3030");
                }
            });
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
