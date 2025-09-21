use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use colony_modsdk::{ModRegistryEntry, HotReloadTransaction, HotReloadStatus, ModLogEntry, LogLevel};
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct ModConsoleState {
    pub selected_mod: Option<String>,
    pub show_logs: bool,
    pub show_docs: bool,
    pub dry_run_ticks: u32,
    pub validation_thresholds: ValidationThresholdsUI,
    pub log_filter: LogFilter,
    pub auto_scroll_logs: bool,
}

#[derive(Default)]
pub struct ValidationThresholdsUI {
    pub max_deadline_hit_rate_change: f32,
    pub max_power_draw_change: f32,
    pub max_bandwidth_util_change: f32,
    pub max_corruption_field_change: f32,
    pub max_heat_level_change: f32,
}

#[derive(Default)]
pub struct LogFilter {
    pub level: Option<LogLevel>,
    pub mod_id: Option<String>,
    pub search_text: String,
}

impl Default for ValidationThresholdsUI {
    fn default() -> Self {
        Self {
            max_deadline_hit_rate_change: 3.0,
            max_power_draw_change: 10.0,
            max_bandwidth_util_change: 5.0,
            max_corruption_field_change: 0.05,
            max_heat_level_change: 5.0,
        }
    }
}

pub fn mod_console_panel(
    mut contexts: EguiContexts,
    mut console_state: ResMut<ModConsoleState>,
    // TODO: Add resources for mod registry, hot reload manager, etc.
) {
    let ctx = contexts.ctx_mut();
    
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Mod Console");
        
        ui.horizontal(|ui| {
            // Mod list
            ui.vertical(|ui| {
                ui.label("Installed Mods");
                
                // Mock mod list - in real implementation, this would come from ModRegistry
                let mock_mods = vec![
                    ("com.example.packetalchemy", "Packet Alchemy", "1.0.0", true),
                    ("com.example.thermalboost", "Thermal Boost", "0.5.0", false),
                    ("com.example.corruptionfix", "Corruption Fix", "2.1.0", true),
                ];
                
                for (id, name, version, enabled) in mock_mods {
                    ui.horizontal(|ui| {
                        let is_selected = console_state.selected_mod.as_ref().map_or(false, |s| s == id);
                        if ui.radio(is_selected, format!("{} v{}", name, version)).clicked() {
                            console_state.selected_mod = Some(id.to_string());
                        }
                        
                        ui.label(if enabled { "✓" } else { "✗" });
                    });
                }
            });
            
            ui.separator();
            
            // Mod details and controls
            ui.vertical(|ui| {
                if let Some(ref mod_id) = console_state.selected_mod {
                    ui.label(format!("Selected: {}", mod_id));
                    
                    // Mod information
                    ui.group(|ui| {
                        ui.label("Mod Information");
                        ui.label("Name: Packet Alchemy");
                        ui.label("Version: 1.0.0");
                        ui.label("Author: Example Corp");
                        ui.label("Description: Advanced packet processing operations");
                        ui.label("Status: Enabled");
                        ui.label("Signature: Valid");
                    });
                    
                    ui.add_space(10.0);
                    
                    // Mod controls
                    ui.group(|ui| {
                        ui.label("Mod Controls");
                        
                        ui.horizontal(|ui| {
                            if ui.button("Enable").clicked() {
                                // TODO: Enable mod
                                println!("Enabling mod: {}", mod_id);
                            }
                            
                            if ui.button("Disable").clicked() {
                                // TODO: Disable mod
                                println!("Disabling mod: {}", mod_id);
                            }
                            
                            if ui.button("Reload").clicked() {
                                // TODO: Hot reload mod
                                println!("Reloading mod: {}", mod_id);
                            }
                        });
                        
                        ui.horizontal(|ui| {
                            if ui.button("View Docs").clicked() {
                                console_state.show_docs = true;
                            }
                            
                            if ui.button("View Logs").clicked() {
                                console_state.show_logs = true;
                            }
                        });
                    });
                    
                    ui.add_space(10.0);
                    
                    // Hot reload settings
                    ui.group(|ui| {
                        ui.label("Hot Reload Settings");
                        
                        ui.horizontal(|ui| {
                            ui.label("Dry Run Ticks:");
                            ui.add(egui::Slider::new(&mut console_state.dry_run_ticks, 60..=300)
                                .text("ticks"));
                        });
                        
                        ui.label("Validation Thresholds:");
                        ui.horizontal(|ui| {
                            ui.label("Deadline Hit Rate:");
                            ui.add(egui::Slider::new(&mut console_state.validation_thresholds.max_deadline_hit_rate_change, 1.0..=10.0)
                                .text("%"));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Power Draw:");
                            ui.add(egui::Slider::new(&mut console_state.validation_thresholds.max_power_draw_change, 5.0..=20.0)
                                .text("%"));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Bandwidth Util:");
                            ui.add(egui::Slider::new(&mut console_state.validation_thresholds.max_bandwidth_util_change, 1.0..=10.0)
                                .text("%"));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Corruption Field:");
                            ui.add(egui::Slider::new(&mut console_state.validation_thresholds.max_corruption_field_change, 0.01..=0.1)
                                .text(""));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Heat Level:");
                            ui.add(egui::Slider::new(&mut console_state.validation_thresholds.max_heat_level_change, 1.0..=10.0)
                                .text("°C"));
                        });
                        
                        if ui.button("Dry Run").clicked() {
                            // TODO: Start dry run
                            println!("Starting dry run for mod: {}", mod_id);
                        }
                    });
                } else {
                    ui.label("Select a mod to view details");
                }
            });
        });
        
        ui.add_space(10.0);
        
        // Hot reload status
        ui.group(|ui| {
            ui.label("Hot Reload Status");
            
            // Mock hot reload transactions
            let mock_transactions = vec![
                ("com.example.packetalchemy", HotReloadStatus::Ready, "Validation passed"),
                ("com.example.thermalboost", HotReloadStatus::ShadowWorld, "Running shadow world..."),
            ];
            
            for (mod_id, status, message) in mock_transactions {
                ui.horizontal(|ui| {
                    ui.label(mod_id);
                    ui.label(format!("{:?}", status));
                    ui.label(message);
                });
            }
        });
        
        ui.add_space(10.0);
        
        // System status
        ui.group(|ui| {
            ui.label("System Status");
            ui.label("Replay Mode: Disabled");
            ui.label("Hot Reload: Enabled");
            ui.label("WASM Host: Running");
            ui.label("Lua Host: Running");
            ui.label("File Watcher: Active");
        });
    });

    // Mod logs window
    if console_state.show_logs {
        mod_logs_window(ctx, &mut console_state);
    }

    // Mod docs window
    if console_state.show_docs {
        mod_docs_window(ctx, &mut console_state);
    }
}

fn mod_logs_window(ctx: &egui::Context, console_state: &mut ModConsoleState) {
    egui::Window::new("Mod Logs")
        .open(&mut console_state.show_logs)
        .show(ctx, |ui| {
            // Log filter
            ui.horizontal(|ui| {
                ui.label("Filter:");
                ui.text_edit_singleline(&mut console_state.log_filter.search_text);
                
                ui.label("Level:");
                egui::ComboBox::from_id_source("log_level")
                    .selected_text(match console_state.log_filter.level {
                        Some(LogLevel::Debug) => "Debug",
                        Some(LogLevel::Info) => "Info",
                        Some(LogLevel::Warn) => "Warn",
                        Some(LogLevel::Error) => "Error",
                        None => "All",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut console_state.log_filter.level, None, "All");
                        ui.selectable_value(&mut console_state.log_filter.level, Some(LogLevel::Debug), "Debug");
                        ui.selectable_value(&mut console_state.log_filter.level, Some(LogLevel::Info), "Info");
                        ui.selectable_value(&mut console_state.log_filter.level, Some(LogLevel::Warn), "Warn");
                        ui.selectable_value(&mut console_state.log_filter.level, Some(LogLevel::Error), "Error");
                    });
                
                ui.checkbox(&mut console_state.auto_scroll_logs, "Auto-scroll");
            });
            
            ui.separator();
            
            // Log entries
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    // Mock log entries
                    let mock_logs = vec![
                        ("com.example.packetalchemy", LogLevel::Info, "Mod loaded successfully"),
                        ("com.example.packetalchemy", LogLevel::Debug, "WASM module initialized"),
                        ("com.example.thermalboost", LogLevel::Warn, "High heat detected"),
                        ("com.example.corruptionfix", LogLevel::Error, "Failed to apply corruption fix"),
                        ("com.example.packetalchemy", LogLevel::Info, "Hot reload completed"),
                    ];
                    
                    for (mod_id, level, message) in mock_logs {
                        // Apply filters
                        if let Some(filter_level) = console_state.log_filter.level {
                            if level != filter_level {
                                continue;
                            }
                        }
                        
                        if !console_state.log_filter.search_text.is_empty() {
                            if !message.to_lowercase().contains(&console_state.log_filter.search_text.to_lowercase()) {
                                continue;
                            }
                        }
                        
                        ui.horizontal(|ui| {
                            // Timestamp
                            ui.label("12:34:56");
                            
                            // Level
                            let level_color = match level {
                                LogLevel::Debug => egui::Color32::GRAY,
                                LogLevel::Info => egui::Color32::WHITE,
                                LogLevel::Warn => egui::Color32::YELLOW,
                                LogLevel::Error => egui::Color32::RED,
                            };
                            ui.colored_label(level_color, format!("{:?}", level));
                            
                            // Mod ID
                            ui.label(format!("[{}]", mod_id));
                            
                            // Message
                            ui.label(message);
                        });
                    }
                });
        });
}

fn mod_docs_window(ctx: &egui::Context, console_state: &mut ModConsoleState) {
    egui::Window::new("Mod Documentation")
        .open(&mut console_state.show_docs)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.heading("Colony Modding API");
                    
                    ui.add_space(10.0);
                    
                    ui.label("WASM Operations");
                    ui.label("WASM operations are sandboxed and deterministic. They can process data and return results.");
                    ui.label("Required exports:");
                    ui.label("  - colony_op_init(ctx: *mut OpCtx) -> i32");
                    ui.label("  - colony_op_process(ctx, input, output, metadata) -> i32");
                    ui.label("  - colony_op_end(ctx: *mut OpCtx) -> i32");
                    
                    ui.add_space(10.0);
                    
                    ui.label("Lua Event Scripts");
                    ui.label("Lua scripts can respond to game events and interact with the simulation.");
                    ui.label("Available functions:");
                    ui.label("  - colony.get_sim_time() -> u64");
                    ui.label("  - colony.get_random() -> u64");
                    ui.label("  - colony.log(level, message)");
                    ui.label("  - colony.get_metric(name) -> f64");
                    ui.label("  - colony.enqueue_job(pipeline_id, payload_size)");
                    
                    ui.add_space(10.0);
                    
                    ui.label("Capabilities");
                    ui.label("Mods must declare capabilities in mod.toml:");
                    ui.label("  - sim_time: Read simulation time");
                    ui.label("  - rng: Access deterministic random number generator");
                    ui.label("  - metrics_read: Read KPI metrics");
                    ui.label("  - enqueue_job: Enqueue jobs into pipelines");
                    ui.label("  - log_debug: Write debug logs");
                    
                    ui.add_space(10.0);
                    
                    ui.label("Security Limits");
                    ui.label("  - WASM fuel limit: 5,000,000 per operation");
                    ui.label("  - WASM memory limit: 64 MB");
                    ui.label("  - Lua instruction budget: 200,000 per tick");
                    ui.label("  - No file system access");
                    ui.label("  - No network access");
                    ui.label("  - No OS system calls");
                });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_console_state_creation() {
        let state = ModConsoleState::default();
        assert!(state.selected_mod.is_none());
        assert!(!state.show_logs);
        assert!(!state.show_docs);
        assert_eq!(state.dry_run_ticks, 0);
        assert!(!state.auto_scroll_logs);
    }

    #[test]
    fn test_validation_thresholds_ui_default() {
        let thresholds = ValidationThresholdsUI::default();
        assert_eq!(thresholds.max_deadline_hit_rate_change, 3.0);
        assert_eq!(thresholds.max_power_draw_change, 10.0);
        assert_eq!(thresholds.max_bandwidth_util_change, 5.0);
        assert_eq!(thresholds.max_corruption_field_change, 0.05);
        assert_eq!(thresholds.max_heat_level_change, 5.0);
    }

    #[test]
    fn test_log_filter_default() {
        let filter = LogFilter::default();
        assert!(filter.level.is_none());
        assert!(filter.mod_id.is_none());
        assert!(filter.search_text.is_empty());
    }

    #[test]
    fn test_mod_console_state_operations() {
        let mut state = ModConsoleState::default();
        
        // Test mod selection
        state.selected_mod = Some("com.test.mymod".to_string());
        assert_eq!(state.selected_mod, Some("com.test.mymod".to_string()));
        
        // Test UI state
        state.show_logs = true;
        state.show_docs = true;
        assert!(state.show_logs);
        assert!(state.show_docs);
        
        // Test configuration
        state.dry_run_ticks = 150;
        assert_eq!(state.dry_run_ticks, 150);
        
        state.auto_scroll_logs = true;
        assert!(state.auto_scroll_logs);
    }
}
