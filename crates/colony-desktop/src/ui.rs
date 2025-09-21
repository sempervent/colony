use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use colony_core::{Colony, Workyard, YardWorkload, thermal_throttle, enqueue_maintenance, JobQueue, IoRuntime, IoRolling, CorruptionField, FaultKpi, ActiveScheduler, SchedPolicy, Worker, WorkerState, GpuFarm, GpuBatchQueues, BlackSwanIndex, Debts, ResearchState, TechTree, WinLossState, SessionCtl, ReplayLog};
use colony_io::IoSimulatorConfig;

mod ui_setup;
mod ui_dashboard;
mod ui_mod_console;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            meters_panel,
            yards_panel,
            tunables_panel,
            io_panel,
            scheduler_panel,
            corruption_panel,
            fault_feed_panel,
            gpu_panel,
            fieldbus_panel,
            events_panel,
            research_panel,
            ui_setup::setup_wizard_panel,
            ui_dashboard::dashboard_panel,
            ui_mod_console::mod_console_panel,
        ));
    }
}

fn meters_panel(
    mut contexts: EguiContexts,
    colony: Res<Colony>,
) {
    egui::SidePanel::left("meters")
        .resizable(true)
        .default_width(200.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("System Meters");
            
            // Power meter
            let power_util = colony.meters.power_draw_kw / colony.power_cap_kw;
            ui.label(format!("Power: {:.0} / {:.0} kW", colony.meters.power_draw_kw, colony.power_cap_kw));
            ui.add(egui::ProgressBar::new(power_util.clamp(0.0, 1.0))
                .text(format!("{:.1}%", power_util * 100.0)));
            
            ui.add_space(10.0);
            
            // Bandwidth meter
            ui.label(format!("Bandwidth: {:.2}", colony.meters.bandwidth_util));
            ui.add(egui::ProgressBar::new(colony.meters.bandwidth_util)
                .text(format!("{:.1}%", colony.meters.bandwidth_util * 100.0)));
            
            ui.add_space(10.0);
            
            // Corruption meter
            ui.label(format!("Corruption: {:.3}", colony.corruption_field));
            ui.add(egui::ProgressBar::new(colony.corruption_field.clamp(0.0, 1.0))
                .text(format!("{:.1}%", colony.corruption_field * 100.0)));
        });
}

fn yards_panel(
    mut contexts: EguiContexts,
    yards: Query<(Entity, &Workyard, &YardWorkload)>,
    colony: Res<Colony>,
    mut jobq: ResMut<JobQueue>,
) {
    egui::SidePanel::right("yards")
        .resizable(true)
        .default_width(250.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Workyards");
            
            for (entity, yard, _workload) in yards.iter() {
                ui.group(|ui| {
                    ui.label(format!("{:?} #{}", yard.kind, entity.index()));
                    
                    // Heat display
                    let heat_util = yard.heat / yard.heat_cap;
                    ui.label(format!("Heat: {:.1}°C / {:.1}°C", yard.heat, yard.heat_cap));
                    ui.add(egui::ProgressBar::new(heat_util.clamp(0.0, 1.0))
                        .text(format!("{:.1}%", heat_util * 100.0)));
                    
                    // Throttle display
                    let throttle = thermal_throttle(
                        yard.heat,
                        yard.heat_cap,
                        colony.tunables.thermal_throttle_knee,
                        colony.tunables.thermal_min_throttle,
                    );
                    ui.label(format!("Throttle: {:.2}x", throttle));
                    
                    // Power draw
                    ui.label(format!("Power: {:.0} kW", yard.power_draw_kw));
                    
                    // Maintenance button
                    if ui.button("Maintenance").clicked() {
                        enqueue_maintenance(entity, &mut jobq);
                    }
                });
                ui.add_space(5.0);
            }
        });
}

fn tunables_panel(
    mut contexts: EguiContexts,
    mut colony: ResMut<Colony>,
) {
    egui::TopBottomPanel::bottom("tunables")
        .resizable(true)
        .default_height(150.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Resource Tunables");
            
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("Power Cap (kW)");
                    ui.add(egui::Slider::new(&mut colony.tunables.power_cap_kw, 100.0..=2000.0));
                });
                
                ui.vertical(|ui| {
                    ui.label("Bandwidth (Gbps)");
                    ui.add(egui::Slider::new(&mut colony.tunables.bandwidth_total_gbps, 1.0..=100.0));
                });
                
                ui.vertical(|ui| {
                    ui.label("Heat Decay/Tick");
                    ui.add(egui::Slider::new(&mut colony.tunables.heat_decay_per_tick, 0.1..=5.0));
                });
            });
            
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("Thermal Knee");
                    ui.add(egui::Slider::new(&mut colony.tunables.thermal_throttle_knee, 0.5..=0.95));
                });
                
                ui.vertical(|ui| {
                    ui.label("Min Throttle");
                    ui.add(egui::Slider::new(&mut colony.tunables.thermal_min_throttle, 0.1..=0.8));
                });
                
                ui.vertical(|ui| {
                    ui.label("Heat Gen/Unit");
                    ui.add(egui::Slider::new(&mut colony.tunables.heat_generated_per_unit, 0.001..=0.1));
                });
            });
        });
}

fn io_panel(
    mut contexts: EguiContexts,
    colony: Res<Colony>,
    io_rolling: Res<IoRolling>,
    jobq: Res<JobQueue>,
) {
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        ui.heading("I/O Control Panel");
        
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label("UDP Simulator");
                if ui.button("Start UDP").clicked() {
                    // TODO: Start UDP simulator
                }
                if ui.button("Stop UDP").clicked() {
                    // TODO: Stop UDP simulator
                }
                
                ui.add_space(10.0);
                
                ui.label("HTTP Simulator");
                if ui.button("Start HTTP").clicked() {
                    // TODO: Start HTTP simulator
                }
                if ui.button("Stop HTTP").clicked() {
                    // TODO: Stop HTTP simulator
                }
            });
            
            ui.vertical(|ui| {
                ui.label("Live Metrics");
                ui.label(format!("Bandwidth: {:.3} Gbps", io_rolling.gbits_this_tick));
                ui.label(format!("Bandwidth Util: {:.1}%", colony.meters.bandwidth_util * 100.0));
                ui.label(format!("Jobs in Queue: {}", jobq.jobs.len()));
                
                ui.add_space(10.0);
                
                ui.label("Pipeline Controls");
                if ui.button("Enqueue UDP Pipeline").clicked() {
                    // TODO: Enqueue UDP pipeline job
                }
                if ui.button("Enqueue HTTP Pipeline").clicked() {
                    // TODO: Enqueue HTTP pipeline job
                }
            });
        });
        
        ui.add_space(20.0);
        
        ui.label("I/O Simulator Configuration");
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label("Rate (Hz)");
                ui.add(egui::Slider::new(&mut 100.0, 1.0..=1000.0));
                
                ui.label("Jitter (ms)");
                ui.add(egui::Slider::new(&mut 5.0, 0.0..=50.0));
            });
            
            ui.vertical(|ui| {
                ui.label("Burstiness");
                ui.add(egui::Slider::new(&mut 0.1, 0.0..=1.0));
                
                ui.label("Loss Rate");
                ui.add(egui::Slider::new(&mut 0.01, 0.0..=0.5));
            });
            
            ui.vertical(|ui| {
                ui.label("Payload Size");
                ui.add(egui::Slider::new(&mut 1024.0, 64.0..=8192.0));
            });
        });
    });
}

fn scheduler_panel(
    mut contexts: EguiContexts,
    mut scheduler: ResMut<ActiveScheduler>,
) {
    egui::SidePanel::right("scheduler_panel").show(contexts.ctx_mut(), |ui| {
        ui.heading("Scheduler Control");
        
        ui.label("Current Policy:");
        ui.label(format!("{}", scheduler.policy));
        
        ui.add_space(10.0);
        
        ui.label("Select Policy:");
        ui.horizontal(|ui| {
            if ui.radio_value(&mut scheduler.policy, SchedPolicy::Fcfs, "FCFS").clicked() {
                // Policy changed
            }
            if ui.radio_value(&mut scheduler.policy, SchedPolicy::Sjf, "SJF").clicked() {
                // Policy changed
            }
            if ui.radio_value(&mut scheduler.policy, SchedPolicy::Edf, "EDF").clicked() {
                // Policy changed
            }
        });
        
        ui.add_space(10.0);
        
        ui.label("Policy Descriptions:");
        ui.label("• FCFS: First Come, First Served");
        ui.label("• SJF: Shortest Job First");
        ui.label("• EDF: Earliest Deadline First");
    });
}

fn corruption_panel(
    mut contexts: EguiContexts,
    colony: Res<Colony>,
    corruption_field: Res<CorruptionField>,
    fault_kpis: Res<FaultKpi>,
    workers: Query<&Worker>,
) {
    egui::SidePanel::left("corruption_panel").show(contexts.ctx_mut(), |ui| {
        ui.heading("Corruption & Faults");
        
        ui.label("Global Corruption:");
        ui.add(egui::ProgressBar::new(corruption_field.global));
        ui.label(format!("{:.3}", corruption_field.global));
        
        ui.add_space(10.0);
        
        // Calculate average worker corruption
        let avg_worker_corruption = if !workers.is_empty() {
            workers.iter().map(|w| w.corruption).sum::<f32>() / workers.iter().count() as f32
        } else {
            0.0
        };
        
        ui.label("Average Worker Corruption:");
        ui.add(egui::ProgressBar::new(avg_worker_corruption));
        ui.label(format!("{:.3}", avg_worker_corruption));
        
        ui.add_space(10.0);
        
        ui.label("Fault Statistics:");
        ui.label(format!("Sticky Workers: {}", fault_kpis.sticky_workers));
        ui.label(format!("Total Faults: {}", fault_kpis.total_faults));
        ui.label(format!("Soft Drop Rate: {:.1}%", fault_kpis.soft_drop_rate * 100.0));
        ui.label(format!("Deadline Hit Rate: {:.1}%", fault_kpis.deadline_hit_rate * 100.0));
        
        ui.add_space(10.0);
        
        ui.label("Fault Breakdown:");
        ui.label(format!("Transient: {}", fault_kpis.transient_faults));
        ui.label(format!("Data Skew: {}", fault_kpis.data_skew_faults));
        ui.label(format!("Sticky Config: {}", fault_kpis.sticky_faults));
        ui.label(format!("Queue Drop: {}", fault_kpis.queue_drop_faults));
        
        ui.add_space(10.0);
        
        ui.label("Corruption Tunables:");
        ui.add(egui::Slider::new(&mut colony.corruption_tun.base_fault_rate, 0.0001..=0.01));
        ui.label("Base Fault Rate");
        
        ui.add(egui::Slider::new(&mut colony.corruption_tun.decay_per_tick, 0.0001..=0.01));
        ui.label("Decay Per Tick");
        
        ui.add(egui::Slider::new(&mut colony.corruption_tun.worker_decay_per_tick, 0.001..=0.02));
        ui.label("Worker Decay Per Tick");
        
        ui.add(egui::Slider::new(&mut colony.corruption_tun.max_retries, 0..=5));
        ui.label("Max Retries");
        
        ui.add(egui::Slider::new(&mut colony.corruption_tun.retry_backoff_ms, 1..=50));
        ui.label("Retry Backoff (ms)");
    });
}

fn fault_feed_panel(
    mut contexts: EguiContexts,
    fault_kpis: Res<FaultKpi>,
) {
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        ui.heading("Fault Feed");
        
        ui.label("Recent Fault Activity:");
        ui.label(format!("Last Tick Faults: {}", fault_kpis.last_tick_faults));
        
        ui.add_space(10.0);
        
        ui.label("Fault KPIs:");
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label("Transient Faults");
                ui.label(format!("{}", fault_kpis.transient_faults));
            });
            
            ui.vertical(|ui| {
                ui.label("Data Skew Faults");
                ui.label(format!("{}", fault_kpis.data_skew_faults));
            });
            
            ui.vertical(|ui| {
                ui.label("Sticky Config Faults");
                ui.label(format!("{}", fault_kpis.sticky_faults));
            });
            
            ui.vertical(|ui| {
                ui.label("Queue Drop Faults");
                ui.label(format!("{}", fault_kpis.queue_drop_faults));
            });
        });
        
        ui.add_space(10.0);
        
        ui.label("Performance Metrics:");
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label("Soft Drop Rate");
                ui.add(egui::ProgressBar::new(fault_kpis.soft_drop_rate));
                ui.label(format!("{:.1}%", fault_kpis.soft_drop_rate * 100.0));
            });
            
            ui.vertical(|ui| {
                ui.label("Deadline Hit Rate");
                ui.add(egui::ProgressBar::new(fault_kpis.deadline_hit_rate));
                ui.label(format!("{:.1}%", fault_kpis.deadline_hit_rate * 100.0));
            });
        });
    });
}

fn gpu_panel(
    mut contexts: EguiContexts,
    gpu_farms: Query<&GpuFarm>,
    batch_queues: Res<GpuBatchQueues>,
) {
    egui::SidePanel::right("gpu_panel").show(contexts.ctx_mut(), |ui| {
        ui.heading("GPU Farm Control");
        
        for gpu_farm in gpu_farms.iter() {
            ui.label("GPU Utilization:");
            ui.add(egui::ProgressBar::new(gpu_farm.meters.util));
            ui.label(format!("{:.1}%", gpu_farm.meters.util * 100.0));
            
            ui.add_space(10.0);
            
            ui.label("VRAM Usage:");
            ui.add(egui::ProgressBar::new(gpu_farm.meters.vram_used_gb / gpu_farm.per_gpu.vram_gb));
            ui.label(format!("{:.2} / {:.1} GB", gpu_farm.meters.vram_used_gb, gpu_farm.per_gpu.vram_gb));
            
            ui.add_space(10.0);
            
            ui.label("Batches In Flight:");
            ui.label(format!("{}", gpu_farm.meters.batches_inflight));
            
            ui.label("Batch Latency (EWMA):");
            ui.label(format!("{:.1} ms", gpu_farm.meters.batch_latency_ms));
            
            ui.add_space(10.0);
            
            ui.label("GPU Flags:");
            ui.checkbox(&mut gpu_farm.flags.mixed_precision, "Mixed Precision");
            
            ui.add_space(10.0);
            
            ui.label("GPU Tunables:");
            ui.add(egui::Slider::new(&mut gpu_farm.per_gpu.batch_max, 1..=64));
            ui.label("Batch Max");
            
            ui.add(egui::Slider::new(&mut gpu_farm.per_gpu.batch_timeout_ms, 1..=50));
            ui.label("Batch Timeout (ms)");
            
            ui.add(egui::Slider::new(&mut gpu_farm.per_gpu.pcie_gbps, 1.0..=32.0));
            ui.label("PCIe Bandwidth (Gbps)");
            
            ui.add(egui::Slider::new(&mut gpu_farm.per_gpu.vram_gb, 4.0..=32.0));
            ui.label("VRAM (GB)");
            
            ui.add_space(10.0);
            
            ui.label("Batch Queue Depths:");
            for (pipeline_id, buffer) in &batch_queues.buffers {
                ui.label(format!("{}: {} items", pipeline_id, buffer.items.len()));
            }
        }
    });
}

fn fieldbus_panel(
    mut contexts: EguiContexts,
) {
    egui::SidePanel::left("fieldbus_panel").show(contexts.ctx_mut(), |ui| {
        ui.heading("Fieldbus Control");
        
        ui.label("CAN Bus Simulator");
        ui.horizontal(|ui| {
            if ui.button("Start CAN").clicked() {
                // TODO: Start CAN simulator
            }
            if ui.button("Stop CAN").clicked() {
                // TODO: Stop CAN simulator
            }
        });
        
        ui.add_space(10.0);
        
        ui.label("CAN Configuration:");
        ui.add(egui::Slider::new(&mut 50.0, 1.0..=1000.0));
        ui.label("Rate (Hz)");
        
        ui.add(egui::Slider::new(&mut 2.0, 0.0..=20.0));
        ui.label("Jitter (ms)");
        
        ui.add(egui::Slider::new(&mut 0.1, 0.0..=1.0));
        ui.label("Burstiness");
        
        ui.add(egui::Slider::new(&mut 0.01, 0.0..=0.1));
        ui.label("Error Rate");
        
        ui.add_space(10.0);
        
        ui.label("Modbus Simulator");
        ui.horizontal(|ui| {
            if ui.button("Start Modbus").clicked() {
                // TODO: Start Modbus simulator
            }
            if ui.button("Stop Modbus").clicked() {
                // TODO: Stop Modbus simulator
            }
        });
        
        ui.add_space(10.0);
        
        ui.label("Modbus Configuration:");
        ui.add(egui::Slider::new(&mut 10.0, 1.0..=100.0));
        ui.label("Rate (Hz)");
        
        ui.add(egui::Slider::new(&mut 0.02, 0.0..=0.2));
        ui.label("Loss Rate");
        
        ui.add(egui::Slider::new(&mut 5.0, 0.0..=50.0));
        ui.label("Jitter (ms)");
        
        ui.add(egui::Slider::new(&mut 256.0, 64.0..=2048.0));
        ui.label("Payload Size");
        
        ui.add_space(10.0);
        
        ui.label("Live Metrics:");
        ui.label("CAN PPS: 45.2");
        ui.label("CAN Errors: 2");
        ui.label("Modbus Req/s: 8.1");
        ui.label("Modbus Timeouts: 0");
    });
}

fn events_panel(
    mut contexts: EguiContexts,
    black_swan_index: Res<BlackSwanIndex>,
    debts: Res<Debts>,
) {
    egui::SidePanel::right("events_panel").show(contexts.ctx_mut(), |ui| {
        ui.heading("Black Swan Events");
        
        ui.label("Active Events:");
        for active_id in &black_swan_index.meters.active {
            ui.label(format!("• {}", active_id));
        }
        
        ui.add_space(10.0);
        
        ui.label("Recently Fired:");
        for (id, tick) in &black_swan_index.meters.recently_fired {
            ui.label(format!("• {} (tick {})", id, tick));
        }
        
        ui.add_space(10.0);
        
        ui.label("Active Debts:");
        for debt in &debts.active {
            match debt {
                colony_core::Debt::PowerMult { mult, until_tick } => {
                    ui.label(format!("Power Mult: {:.2}x (until {})", mult, until_tick));
                }
                colony_core::Debt::HeatAdd { celsius, until_tick } => {
                    ui.label(format!("Heat Add: +{:.1}°C (until {})", celsius, until_tick));
                }
                colony_core::Debt::BandwidthTax { mult, until_tick } => {
                    ui.label(format!("Bandwidth Tax: {:.2}x (until {})", mult, until_tick));
                }
                colony_core::Debt::VramLeak { mb_per_tick, until_tick } => {
                    ui.label(format!("VRAM Leak: {:.1} MB/tick (until {})", mb_per_tick, until_tick));
                }
                colony_core::Debt::FaultBias { kind, weight_mult, until_tick } => {
                    ui.label(format!("Fault Bias: {} {:.2}x (until {})", kind, weight_mult, until_tick));
                }
                colony_core::Debt::Illusion { metric, delta, until_tick } => {
                    ui.label(format!("Illusion: {} {:.2} (until {})", metric, delta, until_tick));
                }
            }
        }
        
        ui.add_space(10.0);
        
        ui.label("Available Events:");
        for def in &black_swan_index.defs {
            ui.collapsing(format!("{} - {}", def.id, def.name), |ui| {
                ui.label(format!("Weight: {}", def.weight));
                ui.label(format!("Cooldown: {} ms", def.cooldown_ms));
                ui.label("Triggers:");
                for trigger in &def.triggers {
                    ui.label(format!("  {} {} {} ({} ms)", trigger.metric, trigger.op, trigger.value, trigger.window_ms));
                }
                ui.label("Effects:");
                for effect in &def.effects {
                    ui.label(format!("  {:?}", effect));
                }
            });
        }
    });
}

fn research_panel(
    mut contexts: EguiContexts,
    mut research_state: ResMut<ResearchState>,
    tech_tree: Res<TechTree>,
) {
    egui::SidePanel::left("research_panel").show(contexts.ctx_mut(), |ui| {
        ui.heading("Research & Tech Tree");
        
        ui.label(format!("Research Points: {}", research_state.pts));
        
        ui.add_space(10.0);
        
        ui.label("Available Techs:");
        for tech in tech_tree.get_available_techs(&research_state) {
            ui.collapsing(format!("{} - {} pts", tech.name, tech.cost_pts), |ui| {
                ui.label(&tech.desc);
                ui.label("Prerequisites:");
                for req in &tech.requires {
                    ui.label(format!("  • {}", req));
                }
                ui.label("Grants:");
                for grant in &tech.grants {
                    ui.label(format!("  • {:?}", grant));
                }
                if ui.button("Research").clicked() {
                    research_state.research_tech(tech);
                }
            });
        }
        
        ui.add_space(10.0);
        
        ui.label("Researched Techs:");
        for tech in tech_tree.get_researched_techs(&research_state) {
            ui.label(format!("✅ {}", tech.name));
        }
        
        ui.add_space(10.0);
        
        ui.label("Available Rituals:");
        for ritual in research_state.get_available_rituals() {
            ui.collapsing(&ritual.name, |ui| {
                ui.label(format!("Time: {} ms", ritual.time_ms));
                ui.label(format!("Parts: {}", ritual.parts));
                ui.label("Effects:");
                for effect in &ritual.effects {
                    ui.label(format!("  • {}", effect));
                }
                if ui.button("Start Ritual").clicked() {
                    // TODO: Start ritual
                    println!("Starting ritual: {}", ritual.id);
                }
            });
        }
    });
}
