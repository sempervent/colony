use colony_mod::{ModContent, PipelineDef, BlackSwanEvent, TechDef};

pub fn get_vanilla_content() -> ModContent {
    ModContent {
        pipelines: get_vanilla_pipelines(),
        events: get_vanilla_events(),
        tech: get_vanilla_tech(),
    }
}

fn get_vanilla_pipelines() -> Vec<PipelineDef> {
    vec![
        PipelineDef {
            id: "udp_telemetry_ingest".to_string(),
            ops: vec!["UdpDemux".to_string(), "Decode".to_string(), "Kalman".to_string(), "Export".to_string()],
            qos: "Balanced".to_string(),
            deadline_ms: 50,
            payload_sz: 4096,
        },
        PipelineDef {
            id: "http_api_processing".to_string(),
            ops: vec!["HttpParse".to_string(), "Decode".to_string(), "Fft".to_string()],
            qos: "Latency".to_string(),
            deadline_ms: 100,
            payload_sz: 8192,
        },
        PipelineDef {
            id: "can_bus_monitoring".to_string(),
            ops: vec!["CanParse".to_string(), "Crc".to_string(), "Kalman".to_string()],
            qos: "Throughput".to_string(),
            deadline_ms: 10,
            payload_sz: 64,
        },
    ]
}

fn get_vanilla_events() -> Vec<BlackSwanEvent> {
    vec![
        BlackSwanEvent {
            id: "vram_ecc_propagation".to_string(),
            name: "VRAM: Snow of Ash".to_string(),
            triggers: vec![
                "bandwidth_util>0.95,window=5".to_string(),
                "gpu_thermal_events>=3,window=3600s".to_string(),
                "corruption_field>0.6".to_string(),
            ],
            effects: vec![
                "pipeline.insert=CRC:all_outbound".to_string(),
                "debt.power_mult=1.08,duration=7d".to_string(),
                "ui.illusion=temperature,-5C,12h".to_string(),
            ],
            cure: Some("maintenance.run=memtest_vram,parts=3,time=8h".to_string()),
            weight: 1.0,
        },
        BlackSwanEvent {
            id: "thermal_cascade".to_string(),
            name: "Thermal Cascade".to_string(),
            triggers: vec![
                "heat>0.9,window=10".to_string(),
                "power_draw>0.95".to_string(),
            ],
            effects: vec![
                "throttle.all=0.5".to_string(),
                "corruption_field+=0.2".to_string(),
            ],
            cure: Some("maintenance.run=cooling_cycle,time=2h".to_string()),
            weight: 0.8,
        },
    ]
}

fn get_vanilla_tech() -> Vec<TechDef> {
    vec![
        TechDef {
            id: "truth_beacon".to_string(),
            name: "Truth Beacon".to_string(),
            description: "Reveals true system metrics, countering UI illusions".to_string(),
            cost: 100,
            prerequisites: vec![],
            effects: vec!["ui.illusion_resistance=1.0".to_string()],
        },
        TechDef {
            id: "numa_isolation".to_string(),
            name: "NUMA Isolation".to_string(),
            description: "Isolates workloads to specific NUMA domains".to_string(),
            cost: 200,
            prerequisites: vec!["truth_beacon".to_string()],
            effects: vec!["isolation_domains=4".to_string(), "corruption_resistance=0.3".to_string()],
        },
        TechDef {
            id: "dual_run_adjudicator".to_string(),
            name: "Dual-Run Adjudicator".to_string(),
            description: "Runs critical pipelines twice and compares results".to_string(),
            cost: 300,
            prerequisites: vec!["numa_isolation".to_string()],
            effects: vec!["accuracy_boost=0.5".to_string(), "latency_penalty=2.0".to_string()],
        },
    ]
}
