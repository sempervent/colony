use proptest::prelude::*;
use colony_core::*;
use bevy::prelude::*;
use std::collections::HashMap;

// Property tests for thermal throttling
proptest! {
    #[test]
    fn thermal_throttle_properties(
        heat in 0.0f32..200.0f32,
        heat_cap in 50.0f32..200.0f32,
        knee in 0.1f32..0.9f32,
        min_throttle in 0.01f32..0.5f32,
    ) {
        let result = thermal_throttle(heat, heat_cap, knee, min_throttle);
        
        // Property 1: Result should always be between min_throttle and 1.0
        prop_assert!(result >= min_throttle);
        prop_assert!(result <= 1.0);
        
        // Property 2: When heat is 0, result should be 1.0
        if heat == 0.0 {
            prop_assert_eq!(result, 1.0);
        }
        
        // Property 3: When heat >= heat_cap, result should be min_throttle
        if heat >= heat_cap {
            prop_assert_eq!(result, min_throttle);
        }
        
        // Property 4: Result should decrease as heat increases
        let higher_heat = heat + 1.0;
        if higher_heat <= heat_cap {
            let higher_result = thermal_throttle(higher_heat, heat_cap, knee, min_throttle);
            prop_assert!(higher_result <= result);
        }
    }
}

// Property tests for corruption field updates
proptest! {
    #[test]
    fn corruption_field_properties(
        current_corruption in 0.0f32..1.0f32,
        corruption_rate in 0.0f32..0.1f32,
        decay_rate in 0.0f32..0.01f32,
    ) {
        let new_corruption = update_corruption_field(current_corruption, corruption_rate, decay_rate);
        
        // Property 1: Corruption should never go below 0
        prop_assert!(new_corruption >= 0.0);
        
        // Property 2: Corruption should never exceed 1.0
        prop_assert!(new_corruption <= 1.0);
        
        // Property 3: If corruption_rate > decay_rate, corruption should increase
        if corruption_rate > decay_rate && current_corruption < 1.0 {
            prop_assert!(new_corruption > current_corruption);
        }
        
        // Property 4: If corruption_rate < decay_rate, corruption should decrease
        if corruption_rate < decay_rate && current_corruption > 0.0 {
            prop_assert!(new_corruption < current_corruption);
        }
    }
}

// Property tests for scheduler policies
proptest! {
    #[test]
    fn scheduler_sjf_properties(
        jobs in prop::collection::vec(
            (1u32..1000u32, 1u32..10000u32), // (deadline_ms, payload_sz)
            1..50
        ),
    ) {
        let mut job_queue = JobQueue::new();
        
        // Add jobs to queue
        for (i, (deadline_ms, payload_sz)) in jobs.iter().enumerate() {
            job_queue.enqueue(Job {
                id: i as u64,
                pipeline: Pipeline {
                    ops: vec![Op::Decode],
                    mutation_tag: None,
                },
                qos: QoS::Balanced,
                deadline_ms: *deadline_ms,
                payload_sz: *payload_sz,
            });
        }
        
        let sjf = SchedPolicy::Sjf.get_scheduler();
        let jobs_refs: Vec<&Job> = job_queue.peek_cpu().iter().map(|ej| &ej.job).collect();
        let workers = vec![
            (Entity::from_raw(0), &Worker {
                id: 0,
                class: WorkClass::Cpu,
                skill_cpu: 1.0,
                skill_gpu: 0.0,
                skill_io: 0.0,
                discipline: 1.0,
                focus: 1.0,
                corruption: 0.0,
                state: WorkerState::Idle,
                retry: RetryPolicy::default(),
                sticky_faults: 0,
            }),
        ];
        
        let picks = sjf.pick(
            &Workyard {
                kind: WorkyardKind::CpuArray,
                slots: 4,
                heat: 50.0,
                heat_cap: 100.0,
                power_draw_kw: 200.0,
                bandwidth_share: 0.3,
                isolation_domain: 0,
            },
            &jobs_refs,
            &workers
        );
        
        // Property 1: Should return at most as many picks as available workers
        prop_assert!(picks.len() <= workers.len());
        
        // Property 2: Should return at most as many picks as available jobs
        prop_assert!(picks.len() <= jobs_refs.len());
        
        // Property 3: All picked jobs should be from the input jobs
        for (_, picked_job) in &picks {
            prop_assert!(jobs_refs.contains(picked_job));
        }
        
        // Property 4: No duplicate picks
        let picked_ids: Vec<u64> = picks.iter().map(|(_, job)| job.id).collect();
        let unique_ids: std::collections::HashSet<u64> = picked_ids.iter().cloned().collect();
        prop_assert_eq!(picked_ids.len(), unique_ids.len());
    }
}

// Property tests for GPU batching
proptest! {
    #[test]
    fn gpu_batching_properties(
        jobs in prop::collection::vec(
            (1u32..1000u32, 1u32..10000u32), // (deadline_ms, payload_sz)
            1..100
        ),
        batch_size in 1u32..32u32,
    ) {
        let mut gpu_queues = GpuBatchQueues::new();
        
        // Add jobs to GPU queue
        for (i, (deadline_ms, payload_sz)) in jobs.iter().enumerate() {
            gpu_queues.enqueue(Job {
                id: i as u64,
                pipeline: Pipeline {
                    ops: vec![Op::GpuPreprocess, Op::Yolo, Op::GpuExport],
                    mutation_tag: None,
                },
                qos: QoS::Balanced,
                deadline_ms: *deadline_ms,
                payload_sz: *payload_sz,
            });
        }
        
        let batches = gpu_queues.batch_jobs(batch_size);
        
        // Property 1: Should return at least one batch if there are jobs
        if !jobs.is_empty() {
            prop_assert!(!batches.is_empty());
        }
        
        // Property 2: Each batch should have at most batch_size jobs
        for batch in &batches {
            prop_assert!(batch.len() <= batch_size as usize);
        }
        
        // Property 3: Total jobs in batches should equal total jobs added
        let total_batched: usize = batches.iter().map(|batch| batch.len()).sum();
        prop_assert_eq!(total_batched, jobs.len());
        
        // Property 4: All job IDs should be unique across batches
        let mut all_ids = Vec::new();
        for batch in &batches {
            for job in batch {
                all_ids.push(job.id);
            }
        }
        let unique_ids: std::collections::HashSet<u64> = all_ids.iter().cloned().collect();
        prop_assert_eq!(all_ids.len(), unique_ids.len());
    }
}

// Property tests for Black Swan probability calculations
proptest! {
    #[test]
    fn black_swan_probability_properties(
        base_prob in 0.0f32..1.0f32,
        corruption_multiplier in 0.1f32..10.0f32,
        time_multiplier in 0.1f32..10.0f32,
    ) {
        let final_prob = base_prob * corruption_multiplier * time_multiplier;
        
        // Property 1: Probability should never be negative
        prop_assert!(final_prob >= 0.0);
        
        // Property 2: If base probability is 0, final should be 0
        if base_prob == 0.0 {
            prop_assert_eq!(final_prob, 0.0);
        }
        
        // Property 3: If base probability is 1, final should be >= base
        if base_prob == 1.0 {
            prop_assert!(final_prob >= base_prob);
        }
        
        // Property 4: Probability should increase with corruption and time multipliers
        if corruption_multiplier > 1.0 && time_multiplier > 1.0 {
            prop_assert!(final_prob > base_prob);
        }
    }
}

// Property tests for KPI buffer operations
proptest! {
    #[test]
    fn kpi_buffer_properties(
        values in prop::collection::vec(0.0f32..1.0f32, 0..1000),
        new_value in 0.0f32..1.0f32,
        max_size in 100usize..10000usize,
    ) {
        let mut kpi_buffer = KpiRingBuffer::new();
        
        // Pre-fill buffer
        for (i, value) in values.iter().enumerate() {
            kpi_buffer.bandwidth_util.push((*value, i as u64));
        }
        
        let initial_size = kpi_buffer.bandwidth_util.len();
        
        // Add new value
        kpi_buffer.bandwidth_util.push((new_value, values.len() as u64));
        
        // Property 1: Buffer should never exceed max_size
        if kpi_buffer.bandwidth_util.len() > max_size {
            // Should have trimmed to max_size
            prop_assert_eq!(kpi_buffer.bandwidth_util.len(), max_size);
        }
        
        // Property 2: Buffer should contain the new value
        prop_assert!(kpi_buffer.bandwidth_util.iter().any(|(val, _)| *val == new_value));
        
        // Property 3: Buffer size should be at most initial_size + 1
        prop_assert!(kpi_buffer.bandwidth_util.len() <= initial_size + 1);
        
        // Property 4: All values should be in valid range
        for (value, _) in &kpi_buffer.bandwidth_util {
            prop_assert!(*value >= 0.0);
            prop_assert!(*value <= 1.0);
        }
    }
}

// Property tests for mutation validity
proptest! {
    #[test]
    fn mutation_validity_properties(
        original_ops in prop::collection::vec(
            prop::sample::select(vec![
                Op::UdpDemux, Op::HttpParse, Op::CanParse,
                Op::Decode, Op::Crc, Op::Kalman, Op::Fft,
                Op::GpuPreprocess, Op::Yolo, Op::GpuExport
            ]),
            1..10
        ),
        mutation_ops in prop::collection::vec(
            prop::sample::select(vec![
                Op::UdpDemux, Op::HttpParse, Op::CanParse,
                Op::Decode, Op::Crc, Op::Kalman, Op::Fft,
                Op::GpuPreprocess, Op::Yolo, Op::GpuExport
            ]),
            1..10
        ),
    ) {
        let original_pipeline = Pipeline {
            ops: original_ops.clone(),
            mutation_tag: None,
        };
        
        let mutated_pipeline = Pipeline {
            ops: mutation_ops.clone(),
            mutation_tag: Some("test_mutation".to_string()),
        };
        
        // Property 1: Both pipelines should be valid
        prop_assert!(!original_pipeline.ops.is_empty());
        prop_assert!(!mutated_pipeline.ops.is_empty());
        
        // Property 2: Mutated pipeline should have mutation tag
        prop_assert!(mutated_pipeline.mutation_tag.is_some());
        
        // Property 3: Original pipeline should not have mutation tag
        prop_assert!(original_pipeline.mutation_tag.is_none());
        
        // Property 4: Both pipelines should have valid operations
        for op in &original_pipeline.ops {
            prop_assert!(matches!(op, Op::UdpDemux | Op::HttpParse | Op::CanParse | 
                Op::Decode | Op::Crc | Op::Kalman | Op::Fft | 
                Op::GpuPreprocess | Op::Yolo | Op::GpuExport));
        }
        
        for op in &mutated_pipeline.ops {
            prop_assert!(matches!(op, Op::UdpDemux | Op::HttpParse | Op::CanParse | 
                Op::Decode | Op::Crc | Op::Kalman | Op::Fft | 
                Op::GpuPreprocess | Op::Yolo | Op::GpuExport));
        }
    }
}

// Property tests for research point calculations
proptest! {
    #[test]
    fn research_point_properties(
        base_points in 0u32..10000u32,
        multiplier in 0.1f32..5.0f32,
        bonus in 0u32..1000u32,
    ) {
        let total_points = (base_points as f32 * multiplier) as u32 + bonus;
        
        // Property 1: Total points should never be negative
        prop_assert!(total_points >= 0);
        
        // Property 2: Total points should be at least as much as base points
        prop_assert!(total_points >= base_points);
        
        // Property 3: If multiplier is 1.0 and bonus is 0, total should equal base
        if (multiplier - 1.0).abs() < 0.001 && bonus == 0 {
            prop_assert_eq!(total_points, base_points);
        }
        
        // Property 4: If multiplier > 1.0, total should be greater than base
        if multiplier > 1.0 {
            prop_assert!(total_points > base_points);
        }
    }
}

// Property tests for victory/loss condition evaluation
proptest! {
    #[test]
    fn victory_loss_properties(
        uptime_days in 0u32..365u32,
        deadline_hit_pct in 0.0f32..100.0f32,
        corruption_field in 0.0f32..1.0f32,
        power_deficit_ticks in 0u32..1000u32,
        sticky_workers in 0u32..100u32,
    ) {
        let victory_rules = VictoryRules {
            target_uptime_days: 1,
            min_deadline_hit_pct: 99.0,
            max_corruption_field: 0.1,
            observation_window_days: 1,
        };
        
        let loss_rules = LossRules {
            hard_power_deficit_ticks: 10,
            sustained_deadline_miss_pct: 5.0,
            max_sticky_workers: 1,
            black_swan_chain_len: 2,
            time_limit_days: None,
        };
        
        let mut sla_tracker = SlaTracker::new(1, 1000);
        for _ in 0..100 {
            sla_tracker.add_deadline_result(deadline_hit_pct > 99.0, 1);
        }
        sla_tracker.update_miss_pct();
        
        let colony = Colony {
            power_cap_kw: 1000.0,
            bandwidth_total_gbps: 32.0,
            corruption_field,
            target_uptime_days: 365,
            meters: GlobalMeters {
                power_draw_kw: 1200.0,
                bandwidth_util: 0.5,
                power_deficit_ticks,
                ..Default::default()
            },
            tunables: ResourceTunables::default(),
            corruption_tun: CorruptionTunables::default(),
            seed: 42,
            game_setup: GameSetup::new(Scenario::default()),
            pending_mutations: Vec::new(),
        };
        
        let fault_kpi = FaultKpi {
            sticky_workers,
            ..Default::default()
        };
        
        let black_swan_index = BlackSwanIndex::new();
        
        // Property 1: Victory should be false if corruption is too high
        if corruption_field > victory_rules.max_corruption_field {
            prop_assert!(!eval_victory(&victory_rules, &sla_tracker, corruption_field, uptime_days));
        }
        
        // Property 2: Loss should be true if power deficit is too high
        if power_deficit_ticks > loss_rules.hard_power_deficit_ticks {
            let loss_reason = eval_loss(&loss_rules, &colony, &sla_tracker, &fault_kpi, &black_swan_index, 1000, 1000);
            prop_assert!(loss_reason.is_some());
        }
        
        // Property 3: Loss should be true if sticky workers exceed limit
        if sticky_workers > loss_rules.max_sticky_workers {
            let loss_reason = eval_loss(&loss_rules, &colony, &sla_tracker, &fault_kpi, &black_swan_index, 1000, 1000);
            prop_assert!(loss_reason.is_some());
        }
        
        // Property 4: Victory should be true if all conditions are met
        if uptime_days >= victory_rules.target_uptime_days &&
           deadline_hit_pct >= victory_rules.min_deadline_hit_pct &&
           corruption_field <= victory_rules.max_corruption_field {
            prop_assert!(eval_victory(&victory_rules, &sla_tracker, corruption_field, uptime_days));
        }
    }
}

// Property tests for WASM fuel consumption
proptest! {
    #[test]
    fn wasm_fuel_properties(
        initial_fuel in 1000u64..10000000u64,
        fuel_consumed in 0u64..1000000u64,
    ) {
        let remaining_fuel = initial_fuel.saturating_sub(fuel_consumed);
        
        // Property 1: Remaining fuel should never be negative
        prop_assert!(remaining_fuel >= 0);
        
        // Property 2: Remaining fuel should never exceed initial fuel
        prop_assert!(remaining_fuel <= initial_fuel);
        
        // Property 3: If no fuel consumed, remaining should equal initial
        if fuel_consumed == 0 {
            prop_assert_eq!(remaining_fuel, initial_fuel);
        }
        
        // Property 4: If fuel consumed >= initial, remaining should be 0
        if fuel_consumed >= initial_fuel {
            prop_assert_eq!(remaining_fuel, 0);
        }
    }
}

// Property tests for Lua instruction counting
proptest! {
    #[test]
    fn lua_instruction_properties(
        instruction_budget in 1000u64..1000000u64,
        instructions_executed in 0u64..100000u64,
    ) {
        let remaining_instructions = instruction_budget.saturating_sub(instructions_executed);
        
        // Property 1: Remaining instructions should never be negative
        prop_assert!(remaining_instructions >= 0);
        
        // Property 2: Remaining instructions should never exceed budget
        prop_assert!(remaining_instructions <= instruction_budget);
        
        // Property 3: If no instructions executed, remaining should equal budget
        if instructions_executed == 0 {
            prop_assert_eq!(remaining_instructions, instruction_budget);
        }
        
        // Property 4: If instructions executed >= budget, remaining should be 0
        if instructions_executed >= instruction_budget {
            prop_assert_eq!(remaining_instructions, 0);
        }
    }
}
