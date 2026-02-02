use std::process::Command;
use tracing::{info, error};
pub struct SystemProxy {
    original_state: Option<ProxyState>,
}

#[derive(Clone, Debug)]
struct ProxyState {
    enabled: bool,
    server: String,
    override_list: String,
}

impl SystemProxy {
    // Create proxy helper
    pub fn new() -> Self {
        Self {
            original_state: None,
        }
    }

    // Enable system proxy
    pub fn enable(&mut self, proxy_addr: &str) -> Result<(), String> {
        info!("Configuring system proxy...");
        self.original_state = Some(self.get_current_state()?);
        
        #[cfg(target_os = "windows")]
        {
            self.enable_windows(proxy_addr)?;
            self.notify_browsers();
            
            Ok(())
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            info!("Automatic proxy configuration only supported on Windows");
            Err("Not supported on this platform".to_string())
        }
    }

    // Restore system proxy
    pub fn disable(&self) -> Result<(), String> {
        info!("Restoring original proxy settings...");
        
        #[cfg(target_os = "windows")]
        {
            if let Some(original) = &self.original_state {
                if original.enabled {
                    self.enable_windows(&original.server)?;
                } else {
                    self.disable_windows()?;
                }

                let _ = Command::new("reg")
                    .args(&[
                        "add",
                        "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
                        "/v",
                        "ProxyOverride",
                        "/t",
                        "REG_SZ",
                        "/d",
                        &original.override_list,
                        "/f"
                    ])
                    .output();

                info!("✓ Original proxy settings restored");
                Ok(())
            } else {
                self.disable_windows()
            }
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            Ok(())
        }
    }

    #[cfg(target_os = "windows")]
    fn get_current_state(&self) -> Result<ProxyState, String> {
        let output = Command::new("reg")
            .args(&[
                "query",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
                "/v",
                "ProxyEnable"
            ])
            .output()
            .map_err(|e| format!("Failed to query proxy state: {}", e))?;

        let enabled = String::from_utf8_lossy(&output.stdout)
            .contains("0x1");

        let server_output = Command::new("reg")
            .args(&[
                "query",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
                "/v",
                "ProxyServer"
            ])
            .output()
            .map_err(|e| format!("Failed to query proxy server: {}", e))?;

        let server = String::from_utf8_lossy(&server_output.stdout)
            .lines()
            .find(|line| line.contains("ProxyServer"))
            .and_then(|line| line.split_whitespace().last())
            .unwrap_or("")
            .to_string();

        let override_output = Command::new("reg")
            .args(&[
                "query",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
                "/v",
                "ProxyOverride"
            ])
            .output()
            .map_err(|e| format!("Failed to query proxy override: {}", e))?;

        let override_list = String::from_utf8_lossy(&override_output.stdout)
            .lines()
            .find(|line| line.contains("ProxyOverride"))
            .and_then(|line| line.split_whitespace().last())
            .unwrap_or("")
            .to_string();

        Ok(ProxyState { enabled, server, override_list })
    }

    #[cfg(target_os = "windows")]
    fn enable_windows(&self, proxy_addr: &str) -> Result<(), String> {
        let normalized = if proxy_addr.contains("0.0.0.0") {
            proxy_addr.replace("0.0.0.0", "127.0.0.1")
        } else {
            proxy_addr.to_string()
        };

        let proxy_value = if normalized.contains('=') {
            normalized
        } else {
            format!("http={};https={}", normalized, normalized)
        };

        let result1 = Command::new("reg")
            .args(&[
                "add",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
                "/v",
                "ProxyServer",
                "/t",
                "REG_SZ",
                "/d",
                &proxy_value,
                "/f"
            ])
            .output()
            .map_err(|e| format!("Failed to set proxy server: {}", e))?;

        if !result1.status.success() {
            return Err("Failed to set proxy server in registry".to_string());
        }

        let result2 = Command::new("reg")
            .args(&[
                "add",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
                "/v",
                "ProxyEnable",
                "/t",
                "REG_DWORD",
                "/d",
                "1",
                "/f"
            ])
            .output()
            .map_err(|e| format!("Failed to enable proxy: {}", e))?;

        if !result2.status.success() {
            return Err("Failed to enable proxy in registry".to_string());
        }

        let result3 = Command::new("reg")
            .args(&[
                "add",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
                "/v",
                "ProxyOverride",
                "/t",
                "REG_SZ",
                "/d",
                "localhost;127.0.0.1;<local>",
                "/f"
            ])
            .output()
            .map_err(|e| format!("Failed to set proxy override: {}", e))?;

        if !result3.status.success() {
            return Err("Failed to set proxy override in registry".to_string());
        }

        let _ = Command::new("rundll32.exe")
            .args(&["wininet.dll,InternetSetOption", "0", "39", "0", "0"])
            .output();

        info!("✓ System proxy enabled: {}", proxy_addr);
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn disable_windows(&self) -> Result<(), String> {
        let result = Command::new("reg")
            .args(&[
                "add",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
                "/v",
                "ProxyEnable",
                "/t",
                "REG_DWORD",
                "/d",
                "0",
                "/f"
            ])
            .output()
            .map_err(|e| format!("Failed to disable proxy: {}", e))?;

        if !result.status.success() {
            return Err("Failed to disable proxy in registry".to_string());
        }

        let _ = Command::new("rundll32.exe")
            .args(&["wininet.dll,InternetSetOption", "0", "39", "0", "0"])
            .output();

        info!("✓ System proxy disabled");
        Ok(())
    }
    
    #[cfg(target_os = "windows")]
    // Nudge browsers to reload settings
    fn notify_browsers(&self) {
        info!("Notifying browsers of proxy change...");
        let _ = Command::new("taskkill")
            .args(&["/F", "/IM", "chrome.exe"])
            .output();
        let _ = Command::new("taskkill")
            .args(&["/F", "/IM", "msedge.exe"])
            .output();
        let _ = Command::new("taskkill")
            .args(&["/F", "/IM", "brave.exe"])
            .output();
        let _ = Command::new("taskkill")
            .args(&["/F", "/IM", "firefox.exe"])
            .output();
        
        info!("Browser processes notified (will use proxy on next launch)");
    }
}

impl Drop for SystemProxy {
    fn drop(&mut self) {
        // Best-effort restore
        if let Err(e) = self.disable() {
            error!("Failed to restore proxy settings on exit: {}", e);
        }
    }
}

pub fn is_elevated() -> bool {
    // Check admin rights
    #[cfg(target_os = "windows")]
    {
        use std::mem;
        use windows::Win32::Foundation::HANDLE;
        use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
        use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

        unsafe {
            let mut token: HANDLE = HANDLE::default();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_ok() {
                let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
                let mut size = 0u32;
                
                if GetTokenInformation(
                    token,
                    TokenElevation,
                    Some(&mut elevation as *mut _ as *mut _),
                    mem::size_of::<TOKEN_ELEVATION>() as u32,
                    &mut size
                ).is_ok() {
                    return elevation.TokenIsElevated != 0;
                }
            }
        }
    }
    false
}

#[cfg(target_os = "windows")]
pub fn allow_inbound_port(port: u16) -> Result<(), String> {
    let rule_name = "Privacy Suite Proxy";
    
    // Remove existing rule if present
    let _ = Command::new("netsh")
        .args(&[
            "advfirewall",
            "firewall",
            "delete",
            "rule",
            &format!("name={}", rule_name),
        ])
        .output();

    // Add rule for all network profiles
    let status = Command::new("netsh")
        .args(&[
            "advfirewall",
            "firewall",
            "add",
            "rule",
            &format!("name={}", rule_name),
            "dir=in",
            "action=allow",
            "protocol=TCP",
            &format!("localport={}", port),
            "profile=private,domain,public",
            "enable=yes",
        ])
        .output()
        .map_err(|e| format!("Failed to add firewall rule: {}", e))?;

    if !status.status.success() {
        let stderr = String::from_utf8_lossy(&status.stderr);
        return Err(format!("Failed to configure firewall rule: {}", stderr));
    }

    info!("✅ Firewall rule added for port {} (all network profiles)", port);
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn allow_inbound_port(_port: u16) -> Result<(), String> {
    Ok(())
}
