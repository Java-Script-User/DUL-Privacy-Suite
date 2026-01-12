use eframe::egui;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Debug)]
pub struct AppState {
    pub tor_connected: bool,
    pub kill_switch_active: bool,
    pub requests_blocked: u64,
    pub trackers_blocked: u64,
    pub webrtc_blocked: u64,
    pub ipv6_blocked: u64,
    pub total_requests: u64,
    pub proxy_running: bool,
    pub auto_proxy_enabled: bool,
}

impl AppState {
    pub fn increment_tracker(&mut self) {
        self.trackers_blocked += 1;
        self.requests_blocked += 1;
    }
    
    pub fn increment_webrtc(&mut self) {
        self.webrtc_blocked += 1;
        self.requests_blocked += 1;
    }
    
    pub fn increment_ipv6(&mut self) {
        self.ipv6_blocked += 1;
        self.requests_blocked += 1;
    }
    
    pub fn increment_request(&mut self) {
        self.total_requests += 1;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            tor_connected: false,
            kill_switch_active: true,
            requests_blocked: 0,
            trackers_blocked: 0,
            webrtc_blocked: 0,
            ipv6_blocked: 0,
            total_requests: 0,
            proxy_running: false,
            auto_proxy_enabled: false,
        }
    }
}

pub struct PrivacySuiteApp {
    state: Arc<RwLock<AppState>>,
}

impl PrivacySuiteApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, state: Arc<RwLock<AppState>>) -> Self {
        Self { state }
    }
}

impl eframe::App for PrivacySuiteApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Get current state (use try_read to avoid blocking UI)
        let state = if let Ok(s) = self.state.try_read() {
            s.clone()
        } else {
            AppState::default()
        };

        // Custom colors
        let green = egui::Color32::from_rgb(46, 204, 113);
        let red = egui::Color32::from_rgb(231, 76, 60);
        let blue = egui::Color32::from_rgb(52, 152, 219);
        let orange = egui::Color32::from_rgb(230, 126, 34);
        let purple = egui::Color32::from_rgb(155, 89, 182);
        let dark_bg = egui::Color32::from_gray(25);

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.add_space(8.0);
                    
                    // Compact title
                    ui.horizontal(|ui| {
                        ui.heading(egui::RichText::new("🛡️ Privacy Suite").size(22.0).strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if state.proxy_running {
                                ui.colored_label(green, egui::RichText::new("● LIVE").size(14.0).strong());
                            } else {
                                ui.colored_label(red, egui::RichText::new("● OFF").size(14.0).strong());
                            }
                        });
                    });
                    ui.label(egui::RichText::new("Anonymous Browsing").size(11.0).color(egui::Color32::GRAY));
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Compact connection status
                    egui::Frame::none()
                        .fill(dark_bg)
                        .rounding(6.0)
                        .inner_margin(10.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(egui::RichText::new("Tor").size(12.0).strong());
                                    if state.tor_connected {
                                        ui.colored_label(green, "✓ 3-hop");
                                    } else {
                                        ui.colored_label(red, "✗ Off");
                                    }
                                });
                                
                                ui.separator();
                                
                                ui.vertical(|ui| {
                                    ui.label(egui::RichText::new("Kill Switch").size(12.0).strong());
                                    if state.kill_switch_active {
                                        ui.colored_label(green, "✓ On");
                                    } else {
                                        ui.colored_label(orange, "⚠ Off");
                                    }
                                });
                            });
                        });

                    ui.add_space(10.0);

                    // Compact statistics
                    ui.label(egui::RichText::new("Statistics").size(16.0).strong());
                    ui.add_space(5.0);
                    
                    // Stats grid - 2x2
                    ui.horizontal(|ui| {
                        egui::Frame::none()
                            .fill(blue)
                            .rounding(6.0)
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                ui.vertical_centered(|ui| {
                                    ui.label(egui::RichText::new(format!("{}", state.total_requests))
                                        .size(20.0).strong().color(egui::Color32::WHITE));
                                    ui.label(egui::RichText::new("Requests")
                                        .size(10.0).color(egui::Color32::from_gray(220)));
                                });
                            });
                        
                        ui.add_space(5.0);
                        
                        egui::Frame::none()
                            .fill(red)
                            .rounding(6.0)
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                ui.vertical_centered(|ui| {
                                    ui.label(egui::RichText::new(format!("{}", state.trackers_blocked))
                                        .size(20.0).strong().color(egui::Color32::WHITE));
                                    ui.label(egui::RichText::new("Trackers")
                                        .size(10.0).color(egui::Color32::from_gray(220)));
                                });
                            });
                    });
                    
                    ui.add_space(5.0);
                    
                    ui.horizontal(|ui| {
                        egui::Frame::none()
                            .fill(orange)
                            .rounding(6.0)
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                ui.vertical_centered(|ui| {
                                    ui.label(egui::RichText::new(format!("{}", state.webrtc_blocked))
                                        .size(20.0).strong().color(egui::Color32::WHITE));
                                    ui.label(egui::RichText::new("WebRTC")
                                        .size(10.0).color(egui::Color32::from_gray(220)));
                                });
                            });
                        
                        ui.add_space(5.0);
                        
                        egui::Frame::none()
                            .fill(purple)
                            .rounding(6.0)
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                ui.vertical_centered(|ui| {
                                    ui.label(egui::RichText::new(format!("{}", state.ipv6_blocked))
                                        .size(20.0).strong().color(egui::Color32::WHITE));
                                    ui.label(egui::RichText::new("IPv6")
                                        .size(10.0).color(egui::Color32::from_gray(220)));
                                });
                            });
                    });
                    
                    ui.add_space(5.0);
                    
                    // Total protected - highlight
                    egui::Frame::none()
                        .fill(green)
                        .rounding(6.0)
                        .inner_margin(10.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🛡️").size(24.0));
                                ui.vertical(|ui| {
                                    ui.label(egui::RichText::new(format!("{}", state.requests_blocked))
                                        .size(22.0).strong().color(egui::Color32::WHITE));
                                    ui.label(egui::RichText::new("Total Blocked")
                                        .size(11.0).color(egui::Color32::from_gray(220)));
                                });
                            });
                        });

                    ui.add_space(10.0);

                    // Compact protections
                    ui.label(egui::RichText::new("Active Protections").size(16.0).strong());
                    ui.add_space(5.0);
                    
                    egui::Frame::none()
                        .fill(dark_bg)
                        .rounding(6.0)
                        .inner_margin(10.0)
                        .show(ui, |ui| {
                            egui::Grid::new("protections_grid")
                                .num_columns(2)
                                .spacing([5.0, 3.0])
                                .show(ui, |ui| {
                                    let features = [
                                        ("🔄", "Multi-hop Tor (3)"),
                                        ("🎭", "Fingerprint randomization"),
                                        ("🚫", "Tracker blocking"),
                                        ("🔒", "DNS encryption"),
                                        ("📹", "WebRTC protection"),
                                        ("🌐", "IPv6 prevention"),
                                        ("🎨", "Canvas protection"),
                                        ("⚡", "Kill switch"),
                                    ];
                                    
                                    for (i, (icon, text)) in features.iter().enumerate() {
                                        ui.horizontal(|ui| {
                                            ui.colored_label(green, *icon);
                                            ui.label(egui::RichText::new(*text).size(11.0));
                                        });
                                        if i % 2 == 1 {
                                            ui.end_row();
                                        }
                                    }
                                });
                        });

                    ui.add_space(10.0);

                    // Configuration
                    ui.label(egui::RichText::new("Setup").size(16.0).strong());
                    ui.add_space(5.0);
                    
                    egui::Frame::none()
                        .fill(egui::Color32::from_gray(30))
                        .rounding(6.0)
                        .inner_margin(10.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Proxy:").strong());
                                ui.label(egui::RichText::new("127.0.0.1:8888").monospace().color(blue));
                            });
                            ui.add_space(3.0);
                            if state.auto_proxy_enabled {
                                ui.horizontal(|ui| {
                                    ui.colored_label(green, "✓");
                                    ui.label(egui::RichText::new("Auto-configured (all apps)").size(11.0).color(green));
                                });
                            } else {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("⚠").color(orange));
                                    ui.label(egui::RichText::new("Manual setup required").size(11.0).color(egui::Color32::GRAY));
                                });
                            }
                        });

                    ui.add_space(8.0);

                    // Buttons
                    ui.horizontal(|ui| {
                        let copy_btn = egui::Button::new(egui::RichText::new("📋 Copy").size(12.0))
                            .fill(blue)
                            .min_size(egui::vec2(100.0, 30.0));
                        if ui.add(copy_btn).clicked() {
                            ui.output_mut(|o| o.copied_text = "127.0.0.1:8888".to_string());
                        }
                        
                        let help_btn = egui::Button::new(egui::RichText::new("❓ Help").size(12.0))
                            .fill(egui::Color32::from_gray(50))
                            .min_size(egui::vec2(80.0, 30.0));
                        if ui.add(help_btn).clicked() {
                            // Help text shown below
                        }
                    });

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(5.0);
                    
                    // Footer
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Privacy Suite v0.1.0").size(10.0).color(egui::Color32::GRAY));
                    });
                    
                    ui.add_space(5.0);
                });
        });

        // Request repaint to keep UI updated
        ctx.request_repaint_after(std::time::Duration::from_secs(1));
    }
}

pub async fn run_gui(state: Arc<RwLock<AppState>>) -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([550.0, 750.0])
            .with_min_inner_size([500.0, 600.0])
            .with_max_inner_size([600.0, 900.0])
            .with_resizable(true)
            .with_icon(
                // Load icon if available
                eframe::icon_data::from_png_bytes(&[]).unwrap_or_default()
            ),
        ..Default::default()
    };

    eframe::run_native(
        "Privacy Suite",
        options,
        Box::new(move |cc| Ok(Box::new(PrivacySuiteApp::new(cc, state.clone())))),
    )
}
