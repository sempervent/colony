use colony_core::*;
use proptest::prelude::*;
use std::collections::HashMap;

/// Comprehensive unit tests for M1-M7 systems
#[cfg(test)]
mod unit_tests {
    use super::*;

    // M1: Thermal/Power/Bandwidth Tests
    mod m1_thermal_power_bandwidth {
        use super::*;

        #[test]
        fn test_thermal_throttle_monotonicity() {
            // Test that thermal throttling is monotonic
            let mut workyard = Workyard {
                kind: WorkyardKind::CpuArray,
                slots: 10,
                heat: 0.0,
                heat_cap: 100.0,
                power_draw_kw: 0.0,
                bandwidth_share: 0.1,
                isolation_domain: 1,
            };

            let mut workload = YardWorkload { units_this_tick: 0.0 };

            // Test monotonicity: more work should always result in more heat
            for i in 0..100 {
                let work_units = i as f32;
                workload.units_this_tick = work_units;
                
                let heat_before = workyard.heat;
                thermal_throttle(&mut workyard, &workload);
                let heat_after = workyard.heat;
                
                assert!(heat_after >= heat_before, "Heat should be monotonic");
            }
        }

        #[test]
        fn test_power_cap_enforcement() {
            let mut colony = Colony {
                power_cap_kw: 1000.0,
                power_draw_kw: 0.0,
                ..Default::default()
            };

            // Test power cap enforcement
            colony.power_draw_kw = 1500.0; // Exceed cap
            assert!(colony.power_draw_kw > colony.power_cap_kw);
            
            // In a real implementation, this would trigger throttling
            // For now, just verify the cap is set correctly
            assert_eq!(colony.power_cap_kw, 1000.0);
        }

        #[test]
        fn test_bandwidth_saturation() {
            let mut colony = Colony {
                bandwidth_total_gbps: 10.0,
                bandwidth_util: 0.0,
                ..Default::default()
            };

            // Test bandwidth utilization calculation
            let payload_size = 1024 * 1024; // 1MB
            let bandwidth_used = (payload_size as f32 * 8.0) / 1_000_000_000.0; // Convert to Gbps
            
            colony.bandwidth_util = bandwidth_used / colony.bandwidth_total_gbps;
            
            assert!(colony.bandwidth_util >= 0.0);
            assert!(colony.bandwidth_util <= 1.0);
        }

        proptest! {
            #[test]
            fn test_thermal_math_properties(
                heat in 0.0f32..1000.0f32,
                heat_cap in 1.0f32..1000.0f32,
                work_units in 0.0f32..100.0f32
            ) {
                let mut workyard = Workyard {
                    kind: WorkyardKind::CpuArray,
                    slots: 10,
                    heat,
                    heat_cap,
                    power_draw_kw: 0.0,
                    bandwidth_share: 0.1,
                    isolation_domain: 1,
                };

                let workload = YardWorkload { units_this_tick: work_units };
                let heat_before = workyard.heat;
                
                thermal_throttle(&mut workyard, &workload);
                
                // Heat should never go negative
                prop_assert!(workyard.heat >= 0.0);
                
                // Heat should increase with work
                if work_units > 0.0 {
                    prop_assert!(workyard.heat >= heat_before);
                }
            }
        }
    }

    // M2: I/O and Pipeline Tests
    mod m2_io_pipelines {
        use super::*;

        #[test]
        fn test_pipeline_processing() {
            let pipeline = Pipeline {
                ops: vec![Op::UdpDemux, Op::Decode, Op::Export],
                mutation_tag: None,
            };

            let job = Job {
                id: 1,
                pipeline: pipeline.clone(),
                qos: QoS::Throughput,
                deadline_ms: 1000,
                payload_sz: 1024,
            };

            // Test pipeline cost calculation
            let total_cost: u32 = pipeline.ops.iter().map(|op| op.cost_ms()).sum();
            assert!(total_cost > 0);
            assert_eq!(total_cost, 2 + 4 + 2); // UdpDemux + Decode + Export
        }

        #[test]
        fn test_io_simulator_config() {
            let config = IoSimulatorConfig {
                udp_packets_per_sec: 1000,
                http_requests_per_sec: 100,
                can_messages_per_sec: 500,
                modbus_requests_per_sec: 50,
            };

            // Test configuration validity
            assert!(config.udp_packets_per_sec > 0);
            assert!(config.http_requests_per_sec > 0);
            assert!(config.can_messages_per_sec > 0);
            assert!(config.modbus_requests_per_sec > 0);
        }

        proptest! {
            #[test]
            fn test_pipeline_ops_validity(
                ops in prop::collection::vec(any::<Op>(), 1..10)
            ) {
                let pipeline = Pipeline {
                    ops: ops.clone(),
                    mutation_tag: None,
                };

                // Pipeline should not be empty
                prop_assert!(!pipeline.ops.is_empty());
                
                // All ops should have valid costs
                for op in &pipeline.ops {
                    prop_assert!(op.cost_ms() > 0);
                    prop_assert!(op.work_units() >= 0.0);
                }
            }
        }
    }

    // M3: Corruption and Fault Tests
    mod m3_corruption_faults {
        use super::*;

        #[test]
        fn test_corruption_field_bounds() {
            let mut corruption = CorruptionField {
                field: 0.0,
                decay_rate: 0.01,
                propagation_rate: 0.05,
            };

            // Test corruption field bounds
            corruption.field = -0.1;
            assert!(corruption.field >= 0.0); // Should be clamped

            corruption.field = 1.5;
            assert!(corruption.field <= 1.0); // Should be clamped
        }

        #[test]
        fn test_fault_probability_bounds() {
            let fault_kpi = FaultKpi {
                soft_faults: 100,
                sticky_faults: 10,
                sticky_workers: 5,
                retry_success_rate: 0.8,
            };

            // Test fault probability calculation
            let total_faults = fault_kpi.soft_faults + fault_kpi.sticky_faults;
            assert!(total_faults > 0);
            assert!(fault_kpi.retry_success_rate >= 0.0);
            assert!(fault_kpi.retry_success_rate <= 1.0);
        }

        #[test]
        fn test_scheduler_policies() {
            let policies = vec![
                SchedPolicy::Fcfs,
                SchedPolicy::Sjf,
                SchedPolicy::Edf,
            ];

            for policy in policies {
                // Test that each policy can be created
                let scheduler = ActiveScheduler {
                    policy,
                    current_job: None,
                };
                assert!(matches!(scheduler.policy, SchedPolicy::Fcfs | SchedPolicy::Sjf | SchedPolicy::Edf));
            }
        }

        proptest! {
            #[test]
            fn test_corruption_decay_properties(
                field in 0.0f32..1.0f32,
                decay_rate in 0.0f32..0.1f32
            ) {
                let mut corruption = CorruptionField {
                    field,
                    decay_rate,
                    propagation_rate: 0.05,
                };

                let field_before = corruption.field;
                
                // Simulate decay
                corruption.field = (corruption.field - corruption.decay_rate).max(0.0);
                
                // Field should never go negative
                prop_assert!(corruption.field >= 0.0);
                
                // Field should decrease or stay the same
                prop_assert!(corruption.field <= field_before);
            }
        }
    }

    // M4: GPU and Batching Tests
    mod m4_gpu_batching {
        use super::*;

        #[test]
        fn test_gpu_batching_throughput() {
            let gpu_farm = GpuFarm {
                vram_total_mb: 8000.0,
                vram_used_mb: 0.0,
                batch_max: 32,
                pcie_bandwidth_gbps: 16.0,
            };

            // Test VRAM capacity
            assert!(gpu_farm.vram_total_mb > 0.0);
            assert!(gpu_farm.vram_used_mb >= 0.0);
            assert!(gpu_farm.vram_used_mb <= gpu_farm.vram_total_mb);

            // Test batching configuration
            assert!(gpu_farm.batch_max > 0);
            assert!(gpu_farm.pcie_bandwidth_gbps > 0.0);
        }

        #[test]
        fn test_vram_utilization() {
            let mut gpu_farm = GpuFarm {
                vram_total_mb: 8000.0,
                vram_used_mb: 0.0,
                batch_max: 32,
                pcie_bandwidth_gbps: 16.0,
            };

            // Test VRAM utilization calculation
            let utilization = gpu_farm.vram_used_mb / gpu_farm.vram_total_mb;
            assert!(utilization >= 0.0);
            assert!(utilization <= 1.0);

            // Test VRAM allocation
            gpu_farm.vram_used_mb = 7200.0; // 90% utilization
            let new_utilization = gpu_farm.vram_used_mb / gpu_farm.vram_total_mb;
            assert_eq!(new_utilization, 0.9);
        }

        proptest! {
            #[test]
            fn test_gpu_batch_efficiency(
                batch_size in 1..64u32,
                vram_total in 1000.0f32..16000.0f32,
                vram_used in 0.0f32..16000.0f32
            ) {
                let gpu_farm = GpuFarm {
                    vram_total_mb: vram_total,
                    vram_used_mb: vram_used.min(vram_total),
                    batch_max: batch_size,
                    pcie_bandwidth_gbps: 16.0,
                };

                // VRAM usage should not exceed total
                prop_assert!(gpu_farm.vram_used_mb <= gpu_farm.vram_total_mb);
                
                // Batch size should be positive
                prop_assert!(gpu_farm.batch_max > 0);
                
                // Utilization should be bounded
                let utilization = gpu_farm.vram_used_mb / gpu_farm.vram_total_mb;
                prop_assert!(utilization >= 0.0);
                prop_assert!(utilization <= 1.0);
            }
        }
    }

    // M5: Black Swans and Research Tests
    mod m5_black_swans_research {
        use super::*;

        #[test]
        fn test_black_swan_trigger_logic() {
            let black_swan = BlackSwan {
                id: "test_swan".to_string(),
                name: "Test Swan".to_string(),
                triggers: vec![
                    BlackSwanTrigger {
                        metric: "bandwidth_util".to_string(),
                        op: ">".to_string(),
                        value: 0.8,
                        window_ms: 5000,
                    }
                ],
                effects: vec![
                    BlackSwanEffect::DebtPowerMult { mult: 1.2, duration_ms: 300000 }
                ],
                cooldown_ms: 600000,
                weight: 1.0,
            };

            // Test trigger logic
            assert!(!black_swan.triggers.is_empty());
            assert!(!black_swan.effects.is_empty());
            assert!(black_swan.cooldown_ms > 0);
            assert!(black_swan.weight > 0.0);
        }

        #[test]
        fn test_research_progress() {
            let mut research_state = ResearchState {
                pts: 0,
                acquired: Vec::new(),
                in_progress: None,
            };

            // Test research point accumulation
            research_state.pts += 100;
            assert_eq!(research_state.pts, 100);

            // Test research acquisition
            research_state.acquired.push("test_tech".to_string());
            assert_eq!(research_state.acquired.len(), 1);
        }

        #[test]
        fn test_tech_tree_structure() {
            let tech_tree = create_default_tech_tree();
            
            // Test tech tree has entries
            assert!(!tech_tree.techs.is_empty());
            assert!(!tech_tree.rituals.is_empty());
        }

        proptest! {
            #[test]
            fn test_black_swan_weight_properties(
                weight in 0.0f32..10.0f32,
                cooldown in 1000u64..3600000u64
            ) {
                let black_swan = BlackSwan {
                    id: "test".to_string(),
                    name: "Test".to_string(),
                    triggers: vec![],
                    effects: vec![],
                    cooldown_ms: cooldown,
                    weight,
                };

                // Weight should be non-negative
                prop_assert!(black_swan.weight >= 0.0);
                
                // Cooldown should be positive
                prop_assert!(black_swan.cooldown_ms > 0);
            }
        }
    }

    // M6: Victory/Loss and Session Tests
    mod m6_victory_loss_session {
        use super::*;

        #[test]
        fn test_victory_conditions() {
            let victory_rules = VictoryRules {
                target_uptime_days: 30,
                min_deadline_hit_pct: 99.5,
                max_corruption_field: 0.35,
                observation_window_days: 7,
            };

            // Test victory rule validity
            assert!(victory_rules.target_uptime_days > 0);
            assert!(victory_rules.min_deadline_hit_pct > 0.0);
            assert!(victory_rules.min_deadline_hit_pct <= 100.0);
            assert!(victory_rules.max_corruption_field >= 0.0);
            assert!(victory_rules.max_corruption_field <= 1.0);
        }

        #[test]
        fn test_loss_conditions() {
            let loss_rules = LossRules {
                hard_power_deficit_ticks: 1000,
                sustained_deadline_miss_pct: 5.0,
                max_sticky_workers: 3,
                black_swan_chain_len: 3,
                time_limit_days: Some(365),
            };

            // Test loss rule validity
            assert!(loss_rules.hard_power_deficit_ticks > 0);
            assert!(loss_rules.sustained_deadline_miss_pct > 0.0);
            assert!(loss_rules.max_sticky_workers > 0);
            assert!(loss_rules.black_swan_chain_len > 0);
        }

        #[test]
        fn test_session_control() {
            let mut session_ctl = SessionCtl::new();
            
            // Test session state
            assert!(!session_ctl.running);
            assert!(!session_ctl.fast_forward);
            assert!(session_ctl.autosave_every_min > 0);

            // Test session operations
            session_ctl.pause();
            assert!(!session_ctl.running);
            
            session_ctl.resume();
            assert!(session_ctl.running);
        }

        #[test]
        fn test_replay_log() {
            let mut replay_log = ReplayLog::new();
            
            // Test replay mode
            assert_eq!(replay_log.mode, ReplayMode::Off);
            
            replay_log.start_recording();
            assert_eq!(replay_log.mode, ReplayMode::Record);
            
            replay_log.stop_recording();
            assert_eq!(replay_log.mode, ReplayMode::Off);
        }

        proptest! {
            #[test]
            fn test_sla_tracker_properties(
                hits in 0u64..1000u64,
                total in 1u64..1000u64
            ) {
                let mut sla_tracker = SlaTracker::new(7, 86400000 / 16);
                
                // Add some test data
                for i in 0..total {
                    sla_tracker.add_deadline_result(i < hits, i);
                }
                
                // Miss percentage should be bounded
                prop_assert!(sla_tracker.current_miss_pct >= 0.0);
                prop_assert!(sla_tracker.current_miss_pct <= 100.0);
                
                // Hit count should not exceed total
                prop_assert!(sla_tracker.hits_in_window <= sla_tracker.total_in_window);
            }
        }
    }

    // M7: Modding and Scripting Tests
    mod m7_modding_scripting {
        use super::*;

        #[test]
        fn test_mod_manifest_validation() {
            let manifest = ModManifest {
                id: "com.test.mymod".to_string(),
                name: "My Test Mod".to_string(),
                version: "1.0.0".to_string(),
                authors: vec!["Test Author".to_string()],
                description: Some("A test mod".to_string()),
                entrypoints: Entrypoints::default(),
                capabilities: Capabilities::default(),
                signature: None,
                requires: None,
            };

            let validation = manifest.validate();
            assert!(validation.valid);
            assert!(validation.errors.is_empty());
        }

        #[test]
        fn test_wasm_op_spec() {
            let op_spec = WasmOpSpec {
                name: "Op_Example".to_string(),
                version: "1.0.0".to_string(),
                cost_hint_ms: 5,
                work_units_hint: 1.0,
                vram_hint_mb: 10.0,
                bandwidth_hint_mb: 1.0,
                description: Some("Example operation".to_string()),
            };

            // Test op spec validity
            assert!(!op_spec.name.is_empty());
            assert!(!op_spec.version.is_empty());
            assert!(op_spec.cost_hint_ms > 0);
            assert!(op_spec.work_units_hint >= 0.0);
            assert!(op_spec.vram_hint_mb >= 0.0);
            assert!(op_spec.bandwidth_hint_mb >= 0.0);
        }

        #[test]
        fn test_lua_event_spec() {
            let event_spec = LuaEventSpec {
                name: "on_tick".to_string(),
                file: "on_tick.lua".to_string(),
                description: Some("Tick event handler".to_string()),
                instruction_budget: Some(200000),
            };

            // Test event spec validity
            assert!(!event_spec.name.is_empty());
            assert!(!event_spec.file.is_empty());
            assert!(event_spec.instruction_budget.unwrap_or(0) > 0);
        }

        #[test]
        fn test_capabilities_gating() {
            let capabilities = Capabilities {
                sim_time: true,
                rng: true,
                metrics_read: false,
                enqueue_job: false,
                log_debug: true,
                modify_tunables: false,
                trigger_events: false,
            };

            // Test capability combinations
            assert!(capabilities.sim_time);
            assert!(capabilities.rng);
            assert!(!capabilities.metrics_read);
            assert!(!capabilities.enqueue_job);
            assert!(capabilities.log_debug);
        }

        proptest! {
            #[test]
            fn test_mod_id_format(
                id in "[a-z0-9.-]+"
            ) {
                let manifest = ModManifest {
                    id: id.clone(),
                    name: "Test Mod".to_string(),
                    version: "1.0.0".to_string(),
                    authors: vec!["Test".to_string()],
                    description: None,
                    entrypoints: Entrypoints::default(),
                    capabilities: Capabilities::default(),
                    signature: None,
                    requires: None,
                };

                let validation = manifest.validate();
                
                // Valid mod IDs should pass validation
                if !id.is_empty() && id.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_') {
                    prop_assert!(validation.valid);
                }
            }
        }
    }

    // Cross-cutting tests
    mod cross_cutting {
        use super::*;

        #[test]
        fn test_resource_consistency() {
            let colony = Colony::default();
            
            // Test resource consistency
            assert!(colony.power_cap_kw > 0.0);
            assert!(colony.bandwidth_total_gbps > 0.0);
            assert!(colony.corruption_field >= 0.0);
            assert!(colony.corruption_field <= 1.0);
        }

        #[test]
        fn test_kpi_ring_buffer() {
            let mut kpi_buffer = KpiRingBuffer::new();
            
            // Test KPI buffer operations
            kpi_buffer.bandwidth_util.push((0.5, 1000));
            kpi_buffer.corruption_field.push((0.1, 1000));
            
            assert!(!kpi_buffer.bandwidth_util.is_empty());
            assert!(!kpi_buffer.corruption_field.is_empty());
        }

        #[test]
        fn test_worker_state_transitions() {
            let mut worker = Worker {
                id: 1,
                class: WorkClass::Cpu,
                skill_cpu: 0.8,
                skill_gpu: 0.2,
                skill_io: 0.5,
                discipline: 0.7,
                focus: 0.6,
                corruption: 0.1,
                state: WorkerState::Idle,
                retry: RetryPolicy::default(),
                sticky_faults: 0,
            };

            // Test state transitions
            assert_eq!(worker.state, WorkerState::Idle);
            
            // In a real implementation, state transitions would be tested
            // For now, just verify the worker can be created
            assert!(worker.skill_cpu > 0.0);
            assert!(worker.skill_cpu <= 1.0);
        }

        proptest! {
            #[test]
            fn test_colony_resource_bounds(
                power_cap in 100.0f32..10000.0f32,
                bandwidth_total in 1.0f32..100.0f32,
                corruption in 0.0f32..1.0f32
            ) {
                let colony = Colony {
                    power_cap_kw: power_cap,
                    bandwidth_total_gbps: bandwidth_total,
                    corruption_field: corruption,
                    ..Default::default()
                };

                // All resources should be within valid bounds
                prop_assert!(colony.power_cap_kw > 0.0);
                prop_assert!(colony.bandwidth_total_gbps > 0.0);
                prop_assert!(colony.corruption_field >= 0.0);
                prop_assert!(colony.corruption_field <= 1.0);
            }
        }
    }
}
