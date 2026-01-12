use std::process::Command;
use tracing::{info, error};

/// System proxy configuration for Windows
pub struct SystemProxy {
    original_state: Option<ProxyState>,
}

#[derive(Clone, Debug)]
struct ProxyState {
    enabled: bool,
    server: String,
}

impl SystemProxy {
    pub fn new() -> Self {
        Self {
            original_state: None,
        }
    }

    /// Enable system-wide proxy automatically
    pub fn enable(&mut self, proxy_addr: &str) -> Result<(), String> {
        info!("Configuring system proxy...");
        
        // Save current state first
        self.original_state = Some(self.get_current_state()?);
        
        #[cfg(target_os = "windows")]
        {
            // Enable Windows system proxy
            self.enable_windows(proxy_addr)?;
            
            // Also notify browsers to refresh their proxy settings
            self.notify_browsers();
            
            Ok(())
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            info!("Automatic proxy configuration only supported on Windows");
            Err("Not supported on this platform".to_string())
        }
    }

    /// Disable system-wide proxy and restore original settings
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
        // Query current proxy settings from registry
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

        Ok(ProxyState { enabled, server })
    }

    #[cfg(target_os = "windows")]
    fn enable_windows(&self, proxy_addr: &str) -> Result<(), String> {
        // Set proxy server
        let result1 = Command::new("reg")
            .args(&[
                "add",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
                "/v",
                "ProxyServer",
                "/t",
                "REG_SZ",
                "/d",
                proxy_addr,
                "/f"
            ])
            .output()
            .map_err(|e| format!("Failed to set proxy server: {}", e))?;

        if !result1.status.success() {
            return Err("Failed to set proxy server in registry".to_string());
        }

        // Enable proxy
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

        // Refresh settings (trigger Windows to recognize the change)
        let _ = Command::new("rundll32.exe")
            .args(&["wininet.dll,InternetSetOption", "0", "39", "0", "0"])
            .output();

        info!("✓ System proxy enabled: {}", proxy_addr);
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn disable_windows(&self) -> Result<(), String> {
        // Disable proxy
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

        // Refresh settings
        let _ = Command::new("rundll32.exe")
            .args(&["wininet.dll,InternetSetOption", "0", "39", "0", "0"])
            .output();

        info!("✓ System proxy disabled");
        Ok(())
    }
    
    #[cfg(target_os = "windows")]
    fn notify_browsers(&self) {
        // Kill and restart browser processes to force them to pick up new proxy settings
        // This is aggressive but ensures browsers use the proxy
        
        info!("Notifying browsers of proxy change...");
        
        // For Chrome-based browsers (Chrome, Edge, Brave)
        // They read from Windows registry but need a nudge
        let _ = Command::new("taskkill")
            .args(&["/F", "/IM", "chrome.exe"])
            .output();
        let _ = Command::new("taskkill")
            .args(&["/F", "/IM", "msedge.exe"])
            .output();
        let _ = Command::new("taskkill")
            .args(&["/F", "/IM", "brave.exe"])
            .output();
            
        // For Firefox (uses its own proxy settings, but respects system proxy if not overridden)
        let _ = Command::new("taskkill")
            .args(&["/F", "/IM", "firefox.exe"])
            .output();
        
        info!("Browser processes notified (will use proxy on next launch)");
    }
}

impl Drop for SystemProxy {
    fn drop(&mut self) {
        // Automatically restore settings when app closes
        if let Err(e) = self.disable() {
            error!("Failed to restore proxy settings on exit: {}", e);
        }
    }
}

/// Check if running with administrator privileges (required for system proxy)
pub fn is_elevated() -> bool {
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
