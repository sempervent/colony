#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thermal_throttle() {
        // Below knee should return 1.0
        assert_eq!(thermal_throttle(50.0, 100.0, 0.85, 0.4), 1.0);
        
        // At knee should return 1.0
        assert_eq!(thermal_throttle(85.0, 100.0, 0.85, 0.4), 1.0);
        
        // Above knee should throttle
        let throttle = thermal_throttle(90.0, 100.0, 0.85, 0.4);
        assert!(throttle < 1.0);
        assert!(throttle > 0.4); // Should respect floor
        
        // At cap should return floor
        assert_eq!(thermal_throttle(100.0, 100.0, 0.85, 0.4), 0.4);
    }

    #[test]
    fn test_bandwidth_latency_multiplier() {
        // Below 0.7 should return 1.0
        assert_eq!(bandwidth_latency_multiplier(0.5, 2.2), 1.0);
        assert_eq!(bandwidth_latency_multiplier(0.7, 2.2), 1.0);
        
        // Above 0.7 should increase
        let mult = bandwidth_latency_multiplier(0.8, 2.2);
        assert!(mult > 1.0);
        
        // Near saturation should be high
        let mult = bandwidth_latency_multiplier(0.95, 2.2);
        assert!(mult > 2.0);
    }

    #[test]
    fn test_op_costs() {
        assert_eq!(Op::UdpDemux.cost_ms(), 2);
        assert_eq!(Op::Decode.cost_ms(), 4);
        assert_eq!(Op::HttpParse.cost_ms(), 3);
        assert_eq!(Op::Yolo.cost_ms(), 18);
        assert_eq!(Op::MaintenanceCool.cost_ms(), 8);
        
        assert_eq!(Op::UdpDemux.work_units(), 0.5);
        assert_eq!(Op::Decode.work_units(), 1.2);
        assert_eq!(Op::HttpParse.work_units(), 0.6);
        assert_eq!(Op::Yolo.work_units(), 4.5);
        assert_eq!(Op::MaintenanceCool.work_units(), 0.0);
    }

    #[test]
    fn test_io_rolling() {
        let mut rolling = super::IoRolling::default();
        assert_eq!(rolling.gbits_this_tick, 0.0);
        
        rolling.add_bytes(1000);
        assert!(rolling.gbits_this_tick > 0.0);
        
        let value = rolling.take_and_reset();
        assert!(value > 0.0);
        assert_eq!(rolling.gbits_this_tick, 0.0);
    }

    #[test]
    fn test_fault_probability_capped() {
        let tunables = super::CorruptionTunables::default();
        
        // Test that fault probability is capped at 0.35
        let max_prob = super::fault_probability(
            0.1, 1.0, 1.0, 1.0, 1.0, 1.0, &tunables
        );
        
        assert!(max_prob <= 0.35);
    }

    #[test]
    fn test_retry_policy() {
        let mut policy = super::RetryPolicy::default();
        assert_eq!(policy.max_retries, 2);
        
        // Test retry countdown
        policy.max_retries -= 1;
        assert_eq!(policy.max_retries, 1);
        
        policy.max_retries -= 1;
        assert_eq!(policy.max_retries, 0);
    }

    #[test]
    fn test_gpu_ops_vram_requirements() {
        assert!(Op::Yolo.vram_needed_mb(1024) > 0.0);
        assert!(Op::GpuPreprocess.vram_needed_mb(1024) > 0.0);
        assert!(Op::GpuExport.vram_needed_mb(1024) > 0.0);
        assert_eq!(Op::Decode.vram_needed_mb(1024), 0.0);
    }

    #[test]
    fn test_gpu_batch_timing() {
        let tunables = super::GpuTunables::default();
        let flags = super::GpuFlags::default();
        let mut batch = super::GpuBatchBuffer::new();
        
        batch.add_item(super::GpuBatchItem {
            job_id: 1,
            op: Op::Yolo,
            payload_sz: 1024,
            enqueue_tick: 100,
        });
        
        let timing = super::calculate_batch_timing(&batch, &tunables, &flags, true);
        assert!(timing > 0.0);
    }

    #[test]
    fn test_debt_system() {
        let mut debts = super::Debts::new();
        let current_tick = 100;

        debts.add_debt(super::Debt::PowerMult { mult: 1.2, until_tick: 200 });
        debts.add_debt(super::Debt::HeatAdd { celsius: 5.0, until_tick: 200 });

        assert_eq!(debts.get_power_multiplier(current_tick), 1.2);
        assert_eq!(debts.get_heat_addition(current_tick), 5.0);
    }

    #[test]
    fn test_black_swan_trigger_evaluation() {
        let mut black_swan_index = super::BlackSwanIndex::new();
        let mut kpi_buffer = super::KpiRingBuffer::new();
        let current_tick = 1000;

        // Add a Black Swan definition
        let swan_def = super::BlackSwanDef {
            id: "test_swan".to_string(),
            name: "Test Swan".to_string(),
            triggers: vec![
                super::TriggerCond {
                    metric: "bandwidth_util".to_string(),
                    op: ">".to_string(),
                    value: 0.9,
                    window_ms: 5000,
                    count_at_least: None,
                }
            ],
            effects: vec![],
            cure: None,
            weight: 1.0,
            cooldown_ms: 10000,
        };
        black_swan_index.add_black_swan(swan_def);

        // Add some KPI data
        kpi_buffer.add_bandwidth_util(0.95, current_tick - 100);

        let eligible = super::evaluate_triggers(&black_swan_index, &kpi_buffer, current_tick);
        assert!(eligible.contains(&"test_swan".to_string()));
    }

    #[test]
    fn test_research_system() {
        let mut research_state = super::ResearchState::new();
        research_state.pts = 100;

        let tech = super::TechNode {
            id: "test_tech".to_string(),
            name: "Test Tech".to_string(),
            desc: "Test description".to_string(),
            cost_pts: 50,
            requires: vec![],
            grants: vec![],
        };

        assert!(research_state.can_research(&tech));
        assert!(research_state.research_tech(&tech));
        assert!(research_state.has_tech("test_tech"));
        assert_eq!(research_state.pts, 50);
    }

    #[test]
    fn test_game_config_scenarios() {
        let scenarios = super::load_scenarios().unwrap();
        assert!(!scenarios.is_empty());
        
        let first_light = scenarios.iter().find(|s| s.id == "first_light_chill").unwrap();
        assert_eq!(first_light.name, "First Light (Chill)");
        assert_eq!(first_light.difficulty.name, "Chill");
        assert_eq!(first_light.victory.target_uptime_days, 30);
    }

    #[test]
    fn test_victory_evaluation() {
        let victory_rules = super::game_config::VictoryRules::default();
        let mut sla_tracker = super::victory::SlaTracker::new(7, 1000);
        
        // Add some good results
        for _ in 0..100 {
            sla_tracker.add_deadline_result(true, 0);
        }
        
        let corruption_field = 0.2; // Below threshold
        let current_tick = 0;
        let ticks_per_day = 1000;
        
        // Should not be victory yet (need consecutive days)
        assert!(!super::victory::eval_victory(&victory_rules, &sla_tracker, corruption_field, current_tick, ticks_per_day));
    }

    #[test]
    fn test_session_control() {
        let mut session = super::session::SessionCtl::new();
        assert!(session.running);
        assert!(!session.fast_forward);

        session.pause();
        assert!(!session.running);

        session.resume();
        assert!(session.running);

        session.toggle_fast_forward();
        assert!(session.fast_forward);
    }

    #[test]
    fn test_replay_log() {
        let mut log = super::session::ReplayLog::new();
        assert_eq!(log.mode, super::session::ReplayMode::Off);

        log.start_recording();
        assert_eq!(log.mode, super::session::ReplayMode::Record);
        assert!(log.is_recording());

        log.record_event(super::session::ReplayEvent::Tick { n: 100 });
        assert_eq!(log.events.len(), 1);

        log.start_playback();
        assert_eq!(log.mode, super::session::ReplayMode::Playback);
        assert!(log.is_playback());

        let event = log.get_next_event();
        assert!(event.is_some());
        assert_eq!(log.events.len(), 0);
    }

    #[test]
    fn test_save_system() {
        let game_setup = super::game_config::GameSetup::new(
            super::game_config::Scenario {
                id: "test".to_string(),
                name: "Test Scenario".to_string(),
                description: "Test".to_string(),
                seed: 42,
                difficulty: super::game_config::Difficulty::default(),
                victory: super::game_config::VictoryRules::default(),
                loss: super::game_config::LossRules::default(),
                start_tunables: None,
                enabled_pipelines: None,
                enabled_events: None,
            }
        );

        let colony = super::Colony {
            power_cap_kw: 1000.0,
            bandwidth_total_gbps: 32.0,
            corruption_field: 0.1,
            target_uptime_days: 365,
            meters: super::GlobalMeters::new(),
            tunables: super::ResourceTunables::default(),
            corruption_tun: super::CorruptionTunables::default(),
            seed: 42,
        };

        let research_state = super::ResearchState::new();
        let black_swan_state = super::BlackSwanIndex::new();
        let debts = super::Debts::new();
        let winloss = super::victory::WinLossState::new();
        let session_ctl = super::session::SessionCtl::new();
        let replay_log = super::session::ReplayLog::new();
        let kpi_summary = super::save::KpiSummary {
            bandwidth_util_history: vec![0.5, 0.6, 0.7],
            corruption_field_history: vec![0.1, 0.2, 0.3],
            power_draw_history: vec![800.0, 900.0, 1000.0],
            heat_levels_history: vec![50.0, 60.0, 70.0],
            deadline_hit_rates: vec![99.0, 98.5, 99.2],
            black_swan_events: vec![("test_event".to_string(), 1000)],
        };

        let save_data = super::save::SaveFileV1::new(
            game_setup,
            &colony,
            &research_state,
            &black_swan_state,
            &debts,
            &winloss,
            &session_ctl,
            &replay_log,
            kpi_summary,
        );

        assert_eq!(save_data.version, 1);
        assert_eq!(save_data.colony_state.power_cap_kw, 1000.0);
    }

    #[test]
    fn test_wasm_host_creation() {
        let host = super::script::WasmHost::new();
        assert!(host.modules.is_empty());
        assert_eq!(host.execution_env.fuel_limit, 5_000_000);
    }

    #[test]
    fn test_lua_host_creation() {
        let host = super::script::LuaHost::new();
        assert!(host.scripts.is_empty());
        assert_eq!(host.execution_env.instruction_budget, 200_000);
        assert!(host.execution_env.sandbox_mode);
    }

    #[test]
    fn test_mod_loader_creation() {
        let temp_dir = std::path::PathBuf::from("/tmp");
        let loader = super::ModLoader::new(temp_dir);
        assert!(loader.registry.mods.is_empty());
        assert!(loader.registry.hot_reload_queue.is_empty());
    }

    #[test]
    fn test_hot_reload_manager() {
        let manager = super::HotReloadManager::new();
        assert!(manager.active_transactions.is_empty());
        assert!(manager.shadow_world_state.is_none());
        assert_eq!(manager.dry_run_ticks, 120);
    }

    #[test]
    fn test_dynamic_ops() {
        use super::components::Op;
        
        let wasm_op = Op::DynamicWasm { op_id: "Op_Example".to_string() };
        assert_eq!(wasm_op.cost_ms(), 5);
        assert_eq!(wasm_op.work_units(), 1.0);
        
        let lua_op = Op::DynamicLua { func: "on_tick".to_string() };
        assert_eq!(lua_op.cost_ms(), 2);
        assert_eq!(lua_op.work_units(), 0.5);
    }
}

// Include comprehensive test modules
#[cfg(test)]
mod unit_tests;
#[cfg(test)]
mod determinism_tests;
#[cfg(test)]
mod security_tests;
#[cfg(test)]
mod property_tests;
#[cfg(test)]
mod m7_unit_tests;
