use serde::{Deserialize, Serialize};
use tauri::{Manager, menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem}, tray::{TrayIconBuilder, TrayIconEvent}, Emitter};
use tauri::WindowEvent;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
struct Stats {
    tor_connected: bool,
    kill_switch_active: bool,
    requests_blocked: u64,
    trackers_blocked: u64,
    webrtc_blocked: u64,
    ipv6_blocked: u64,
    fingerprints_randomized: u64,
    last_fingerprint_at: Option<u64>,
    last_fingerprint_user_agent: Option<String>,
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
    // Request backend shutdown
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    let _ = client
        .post("http://127.0.0.1:3030/api/shutdown")
        .send()
        .await;
    
    Ok(())
}

#[tauri::command]
async fn close_window_only(app: tauri::AppHandle) -> Result<(), String> {
    // Hide window but keep running
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| format!("Failed to hide window: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
async fn shutdown_and_exit(app: tauri::AppHandle) -> Result<(), String> {
    // Stop backend then exit
    let _ = shutdown_backend().await;
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    app.exit(0);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // Use custom close dialog
                api.prevent_close();
                let window_clone = window.clone();
                println!("Close requested - showing close dialog");
                let _ = window_clone.emit("show-close-dialog", ());
            }
        })
        .invoke_handler(tauri::generate_handler![get_stats, get_logs, toggle_kill_switch, toggle_connection, shutdown_backend, close_window_only, shutdown_and_exit])
        .setup(|app| {
            #[cfg(target_os = "windows")]
            {
                let elevated = Command::new("net")
                    .args(["session"])
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false);
                if !elevated {
                    if let Ok(exe_path) = std::env::current_exe() {
                        let _ = Command::new("powershell")
                            .args([
                                "-Command",
                                &format!("Start-Process -FilePath '{}' -Verb RunAs", exe_path.display()),
                            ])
                            .spawn();
                    }
                    app.handle().exit(0);
                    return Ok(());
                }
            }

            #[cfg(target_os = "windows")]
            {
                // Resources are bundled with the app - just verify they exist
                if let Ok(resource_dir) = app.path().resource_dir() {
                    let backend_path = resource_dir.join("privacy_suite.exe");
                    let lyrebird_path = resource_dir.join("lyrebird.exe");
                    
                    if backend_path.exists() {
                        println!("Found backend at: {:?}", backend_path);
                    } else {
                        eprintln!("WARNING: Backend not found at {:?}", backend_path);
                    }
                    
                    if lyrebird_path.exists() {
                        println!("Found lyrebird at: {:?}", lyrebird_path);
                    } else {
                        eprintln!("WARNING: Lyrebird not found at {:?}", lyrebird_path);
                    }
                }
            }

            // Tray menu
            let show = MenuItemBuilder::with_id("show", "Show Window").build(app)?;
            let separator = tauri::menu::PredefinedMenuItem::separator(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Exit").build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&show)
                .item(&separator)
                .item(&quit)
                .build()?;
            let tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .menu_on_left_click(false)
                .tooltip("Privacy Suite")
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                            let _ = window.unminimize();
                        }
                    }
                    "quit" => {
                        // Graceful backend shutdown
                        println!("Tray quit requested - shutting down backend...");
                        let _ = reqwest::blocking::Client::new()
                            .post("http://127.0.0.1:3030/api/shutdown")
                            .timeout(std::time::Duration::from_secs(2))
                            .send();
                        std::thread::sleep(std::time::Duration::from_millis(500));
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    // Toggle window on left click
                    if let TrayIconEvent::Click { button, .. } = event {
                        if button == tauri::tray::MouseButton::Left {
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                if window.is_visible().unwrap_or(false) {
                                    let _ = window.hide();
                                } else {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                    let _ = window.unminimize();
                                }
                            }
                        }
                    }
                })
                .build(app)?;
            app.manage(tray);

            // Start backend if needed
            let backend_resource_dir = app.path().resource_dir().ok().map(|dir| dir.join("resources"));
            let backend_app_local_dir = app.path().app_local_data_dir().ok();
            
            std::thread::spawn(move || {
                // Debug log file
                let debug_log = backend_app_local_dir
                    .as_ref()
                    .map(|d| d.join("gui_launch_debug.log"))
                    .unwrap_or_else(|| std::env::temp_dir().join("privacy_suite_debug.log"));
                
                let mut log_output = String::new();
                use std::fmt::Write;
                let _ = writeln!(log_output, "=== GUI Launch Debug ===");
                let _ = writeln!(log_output, "Time: {:?}", std::time::SystemTime::now());
                
                let backend_running = std::net::TcpStream::connect("127.0.0.1:3030").is_ok();
                let _ = writeln!(log_output, "Backend already running: {}", backend_running);

                if !backend_running {
                    let _ = writeln!(log_output, "Attempting to launch backend...");

                    // Try resource dir first (where bundled files are in installed app)
                    let _ = writeln!(log_output, "Resource dir: {:?}", backend_resource_dir);
                    
                    let backend_path = backend_resource_dir
                        .as_ref()
                        .map(|dir| dir.join("privacy_suite.exe"))
                        .filter(|p| {
                            let exists = p.exists();
                            let _ = writeln!(log_output, "Checking backend at {:?}: {}", p, exists);
                            exists
                        })
                        .unwrap_or_else(|| std::path::Path::new("privacy_suite.exe").to_path_buf());

                    let lyrebird_path = backend_resource_dir
                        .as_ref()
                        .map(|dir| dir.join("lyrebird.exe"))
                        .filter(|p| {
                            let exists = p.exists();
                            let _ = writeln!(log_output, "Checking lyrebird at {:?}: {}", p, exists);
                            exists
                        })
                        .unwrap_or_else(|| std::path::Path::new("lyrebird.exe").to_path_buf());
                    let log_path = backend_app_local_dir
                        .as_ref()
                        .map(|dir| {
                            let _ = std::fs::create_dir_all(dir);
                            dir.join("backend.log")
                        })
                        .unwrap_or_else(|| std::env::temp_dir().join("privacy_suite_backend.log"));
                    let log_file = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&log_path)
                        .ok();

                    let _ = writeln!(log_output, "Log path: {:?}", log_path);
                    let _ = writeln!(log_output, "Backend path: {:?}", backend_path);
                    let _ = writeln!(log_output, "Lyrebird path: {:?}", lyrebird_path);
                    let _ = writeln!(log_output, "Backend exists: {}", backend_path.exists());

                    if backend_path.exists() {
                        #[cfg(target_os = "windows")]
                        {
                            use std::os::windows::process::CommandExt;
                            const CREATE_NO_WINDOW: u32 = 0x08000000;

                            let backend_abs = backend_path.canonicalize().unwrap_or(backend_path.clone());
                            let lyrebird_abs = lyrebird_path.canonicalize().unwrap_or(lyrebird_path.clone());
                            
                            let _ = writeln!(log_output, "Absolute backend: {:?}", backend_abs);
                            let _ = writeln!(log_output, "Absolute lyrebird: {:?}", lyrebird_abs);
                            
                            let direct = {
                                let mut cmd = std::process::Command::new(&backend_abs);
                                cmd.creation_flags(CREATE_NO_WINDOW)
                                    .env("LYREBIRD_PATH", lyrebird_abs.display().to_string());
                                if let Some(file) = log_file.as_ref() {
                                    let _ = cmd.stdout(file.try_clone().unwrap());
                                    let _ = cmd.stderr(file.try_clone().unwrap());
                                }
                                cmd.spawn()
                            };

                            let result = match direct {
                                Ok(child) => {
                                    let _ = writeln!(log_output, "Backend PID: {:?}", child.id());
                                    Ok(())
                                }
                                Err(e) => {
                                    let _ = writeln!(log_output, "Spawn error: {}", e);
                                    let ps_script = format!(
                                        "$env:LYREBIRD_PATH='{}'; Start-Process -FilePath '{}' -WindowStyle Hidden -Wait:$false",
                                        lyrebird_abs.display().to_string().replace("\\", "\\\\"),
                                        backend_abs.display().to_string().replace("\\", "\\\\")
                                    );
                                    let _ = writeln!(log_output, "PowerShell: {}", ps_script);
                                    std::process::Command::new("powershell")
                                        .args(&[
                                            "-NoProfile",
                                            "-Command",
                                            &ps_script
                                        ])
                                        .creation_flags(CREATE_NO_WINDOW)
                                        .spawn()
                                        .map(|_| ())
                                        .map_err(|e| {
                                            let _ = writeln!(log_output, "PowerShell error: {}", e);
                                            e
                                        })
                                }
                            };

                            match result {
                                Ok(_) => {
                                    let _ = writeln!(log_output, "Waiting...");
                                    for i in 0..30 {
                                        std::thread::sleep(std::time::Duration::from_millis(500));
                                        if std::net::TcpStream::connect("127.0.0.1:3030").is_ok() {
                                            let _ = writeln!(log_output, "Ready after {} attempts", i + 1);
                                            break;
                                        }
                                    }
                                }
                                Err(e) => {
                                    let _ = writeln!(log_output, "Failed: {:?}", e);
                                }
                            }
                        }
                    } else {
                        let _ = writeln!(log_output, "Backend not found!");
                    }
                } else {
                    let _ = writeln!(log_output, "Already running");
                }
                
                // Write debug log
                let _ = std::fs::write(&debug_log, log_output);
            });
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
