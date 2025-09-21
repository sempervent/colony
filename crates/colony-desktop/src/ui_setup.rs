use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use colony_core::{load_scenarios, GameSetup, Scenario, Difficulty, VictoryRules, LossRules};

#[derive(Resource, Default)]
pub struct SetupWizard {
    pub current_step: u32,
    pub selected_scenario: Option<Scenario>,
    pub custom_difficulty: Option<Difficulty>,
    pub custom_victory: Option<VictoryRules>,
    pub custom_loss: Option<LossRules>,
    pub selected_pipelines: Vec<String>,
    pub selected_events: Vec<String>,
    pub tick_scale: String,
    pub seed: u64,
    pub mods: Vec<String>,
}

impl SetupWizard {
    pub fn new() -> Self {
        Self {
            current_step: 0,
            selected_scenario: None,
            custom_difficulty: None,
            custom_victory: None,
            custom_loss: None,
            selected_pipelines: vec![],
            selected_events: vec![],
            tick_scale: "RealTime".to_string(),
            seed: 42,
            mods: vec!["vanilla".to_string()],
        }
    }

    pub fn next_step(&mut self) {
        self.current_step += 1;
    }

    pub fn prev_step(&mut self) {
        if self.current_step > 0 {
            self.current_step -= 1;
        }
    }

    pub fn can_proceed(&self) -> bool {
        match self.current_step {
            0 => self.selected_scenario.is_some(),
            1 => true, // Difficulty adjustments are optional
            2 => true, // Pipeline/event selection is optional
            3 => true, // Time scale and seed are always valid
            _ => false,
        }
    }

    pub fn create_game_setup(&self) -> Option<GameSetup> {
        let scenario = self.selected_scenario.as_ref()?.clone();
        let mut setup = GameSetup::new(scenario);
        
        setup.tick_scale = self.tick_scale.clone();
        setup.mods = self.mods.clone();
        
        Some(setup)
    }
}

pub fn setup_wizard_panel(
    mut contexts: EguiContexts,
    mut wizard: ResMut<SetupWizard>,
    mut commands: Commands,
) {
    let ctx = contexts.ctx_mut();
    
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Colony Simulator Setup");
        
        match wizard.current_step {
            0 => scenario_selection_step(ui, &mut wizard),
            1 => difficulty_adjustment_step(ui, &mut wizard),
            2 => pipeline_event_selection_step(ui, &mut wizard),
            3 => time_scale_seed_step(ui, &mut wizard),
            _ => {
                ui.label("Setup complete!");
                if ui.button("Start Game").clicked() {
                    if let Some(game_setup) = wizard.create_game_setup() {
                        // TODO: Start the game with the selected setup
                        println!("Starting game with setup: {:?}", game_setup);
                        commands.insert_resource(game_setup);
                    }
                }
            }
        }
        
        ui.add_space(20.0);
        
        ui.horizontal(|ui| {
            if ui.button("Back").clicked() {
                wizard.prev_step();
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Next").clicked() && wizard.can_proceed() {
                    wizard.next_step();
                }
            });
        });
    });
}

fn scenario_selection_step(ui: &mut egui::Ui, wizard: &mut SetupWizard) {
    ui.heading("Step 1: Choose Scenario");
    
    if let Ok(scenarios) = load_scenarios() {
        for scenario in scenarios {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    let is_selected = wizard.selected_scenario.as_ref()
                        .map(|s| s.id == scenario.id)
                        .unwrap_or(false);
                    
                    if ui.radio(is_selected, &scenario.name).clicked() {
                        wizard.selected_scenario = Some(scenario.clone());
                    }
                });
                
                ui.label(&scenario.description);
                ui.label(format!("Difficulty: {}", scenario.difficulty.name));
                ui.label(format!("Seed: {}", scenario.seed));
                ui.label(format!("Target: {} days", scenario.victory.target_uptime_days));
            });
        }
    } else {
        ui.label("Failed to load scenarios");
    }
}

fn difficulty_adjustment_step(ui: &mut egui::Ui, wizard: &mut SetupWizard) {
    ui.heading("Step 2: Adjust Difficulty (Optional)");
    
    if let Some(scenario) = &wizard.selected_scenario {
        ui.label(format!("Base scenario: {}", scenario.name));
        ui.label(format!("Difficulty: {}", scenario.difficulty.name));
        
        ui.add_space(10.0);
        
        if ui.button("Use Default Difficulty").clicked() {
            wizard.custom_difficulty = None;
        }
        
        if ui.button("Customize Difficulty").clicked() {
            wizard.custom_difficulty = Some(scenario.difficulty.clone());
        }
        
        if let Some(ref mut difficulty) = wizard.custom_difficulty {
            ui.add_space(10.0);
            ui.label("Custom Difficulty Settings:");
            
            ui.add(egui::Slider::new(&mut difficulty.power_cap_mult, 0.5..=2.0)
                .text("Power Cap Multiplier"));
            ui.add(egui::Slider::new(&mut difficulty.heat_cap_mult, 0.5..=2.0)
                .text("Heat Cap Multiplier"));
            ui.add(egui::Slider::new(&mut difficulty.bw_total_mult, 0.5..=2.0)
                .text("Bandwidth Multiplier"));
            ui.add(egui::Slider::new(&mut difficulty.fault_rate_mult, 0.1..=3.0)
                .text("Fault Rate Multiplier"));
            ui.add(egui::Slider::new(&mut difficulty.black_swan_weight_mult, 0.1..=3.0)
                .text("Black Swan Weight Multiplier"));
            ui.add(egui::Slider::new(&mut difficulty.research_rate_mult, 0.1..=3.0)
                .text("Research Rate Multiplier"));
        }
    }
}

fn pipeline_event_selection_step(ui: &mut egui::Ui, wizard: &mut SetupWizard) {
    ui.heading("Step 3: Select Pipelines & Events (Optional)");
    
    if let Some(scenario) = &wizard.selected_scenario {
        ui.label("Available Pipelines:");
        let all_pipelines = vec![
            "udp_telemetry_ingest".to_string(),
            "http_ingest".to_string(),
            "can_telemetry".to_string(),
            "modbus_poll".to_string(),
        ];
        
        for pipeline in &all_pipelines {
            let is_enabled = scenario.enabled_pipelines.as_ref()
                .map(|p| p.contains(pipeline))
                .unwrap_or(true);
            
            let mut selected = wizard.selected_pipelines.contains(pipeline);
            if ui.checkbox(&mut selected, pipeline).changed() {
                if selected {
                    wizard.selected_pipelines.push(pipeline.clone());
                } else {
                    wizard.selected_pipelines.retain(|p| p != pipeline);
                }
            }
        }
        
        ui.add_space(10.0);
        
        ui.label("Available Black Swan Events:");
        let all_events = vec![
            "vram_ecc_propagation".to_string(),
            "pcie_link_flap".to_string(),
            "clock_skew_bloom".to_string(),
            "packet_monsoon_echo".to_string(),
            "numa_ghosting".to_string(),
            "adjudicator_schism".to_string(),
        ];
        
        for event in &all_events {
            let is_enabled = scenario.enabled_events.as_ref()
                .map(|e| e.contains(event))
                .unwrap_or(true);
            
            let mut selected = wizard.selected_events.contains(event);
            if ui.checkbox(&mut selected, event).changed() {
                if selected {
                    wizard.selected_events.push(event.clone());
                } else {
                    wizard.selected_events.retain(|e| e != event);
                }
            }
        }
    }
}

fn time_scale_seed_step(ui: &mut egui::Ui, wizard: &mut SetupWizard) {
    ui.heading("Step 4: Time Scale & Seed");
    
    ui.label("Time Scale:");
    egui::ComboBox::from_id_source("tick_scale")
        .selected_text(&wizard.tick_scale)
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut wizard.tick_scale, "RealTime".to_string(), "Real Time");
            ui.selectable_value(&mut wizard.tick_scale, "Seconds:1".to_string(), "1 Second per Tick");
            ui.selectable_value(&mut wizard.tick_scale, "Seconds:10".to_string(), "10 Seconds per Tick");
            ui.selectable_value(&mut wizard.tick_scale, "Days:1".to_string(), "1 Day per Tick");
            ui.selectable_value(&mut wizard.tick_scale, "Days:7".to_string(), "7 Days per Tick");
            ui.selectable_value(&mut wizard.tick_scale, "Years:1".to_string(), "1 Year per Tick");
        });
    
    ui.add_space(10.0);
    
    ui.label("Random Seed:");
    ui.add(egui::Slider::new(&mut wizard.seed, 0..=10000)
        .text("Seed"));
    
    if ui.button("Randomize Seed").clicked() {
        wizard.seed = (wizard.seed + 1) % 10000;
    }
    
    ui.add_space(10.0);
    
    ui.label("Loaded Mods:");
    for mod_id in &wizard.mods {
        ui.label(format!("• {}", mod_id));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_wizard_creation() {
        let wizard = SetupWizard::new();
        assert_eq!(wizard.current_step, 0);
        assert!(wizard.selected_scenario.is_none());
        assert_eq!(wizard.tick_scale, "RealTime");
        assert_eq!(wizard.seed, 42);
    }

    #[test]
    fn test_setup_wizard_navigation() {
        let mut wizard = SetupWizard::new();
        assert_eq!(wizard.current_step, 0);
        
        wizard.next_step();
        assert_eq!(wizard.current_step, 1);
        
        wizard.prev_step();
        assert_eq!(wizard.current_step, 0);
        
        wizard.prev_step(); // Should not go below 0
        assert_eq!(wizard.current_step, 0);
    }

    #[test]
    fn test_setup_wizard_validation() {
        let mut wizard = SetupWizard::new();
        assert!(!wizard.can_proceed()); // No scenario selected
        
        // Mock a scenario selection
        let scenario = Scenario {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test".to_string(),
            seed: 42,
            difficulty: Difficulty::default(),
            victory: VictoryRules::default(),
            loss: LossRules::default(),
            start_tunables: None,
            enabled_pipelines: None,
            enabled_events: None,
        };
        wizard.selected_scenario = Some(scenario);
        assert!(wizard.can_proceed());
    }
}
