use super::{IoPacket, IoSimulatorConfig, IoSource};
use bytes::Bytes;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::net::SocketAddr;
use tokio::sync::mpsc;
use tokio::time::{Duration, Instant};

pub struct UdpSimulator {
    config: IoSimulatorConfig,
}

impl UdpSimulator {
    pub fn new(config: IoSimulatorConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl IoSource for UdpSimulator {
    async fn run(self: Box<Self>, tx: mpsc::Sender<IoPacket>, seed: u64) {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut last_packet = Instant::now();
        let mut in_burst = false;
        let mut burst_remaining = 0;
        
        // Poisson inter-arrival time: -ln(U) / rate
        let mean_interval_ms = 1000.0 / self.config.rate_hz;
        
        loop {
            // Check if we should start a burst
            if !in_burst && rng.gen::<f32>() < self.config.burstiness {
                in_burst = true;
                burst_remaining = rng.gen_range(2..=8); // 2-8 packets in burst
            }
            
            // Calculate next packet time
            let interval_ms = if in_burst {
                // Shorter intervals during burst
                mean_interval_ms * 0.1
            } else {
                // Poisson inter-arrival
                -rng.gen::<f32>().ln() * mean_interval_ms
            };
            
            let jitter_ms = rng.gen_range(0..=self.config.jitter_ms) as f32;
            let total_delay = interval_ms + jitter_ms;
            
            tokio::time::sleep(Duration::from_millis(total_delay as u64)).await;
            
            // Simulate packet loss
            if rng.gen::<f32>() < self.config.loss {
                continue;
            }
            
            // Generate packet
            let now = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64;
            let src = SocketAddr::new(
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 1, 100)),
                12345,
            );
            
            // Generate telemetry-like payload
            let payload = format!(
                r#"{{"timestamp":{},"cpu_usage":{:.2},"memory_usage":{:.2},"temperature":{:.1},"load":{:.2}}}"#,
                now,
                rng.gen::<f32>() * 100.0,
                rng.gen::<f32>() * 100.0,
                20.0 + rng.gen::<f32>() * 60.0,
                rng.gen::<f32>() * 10.0,
            );
            
            let data = Bytes::from(payload);
            
            let packet = IoPacket::Udp {
                ts_ns: now,
                src,
                data,
            };
            
            if tx.send(packet).await.is_err() {
                break; // Channel closed
            }
            
            // Update burst state
            if in_burst {
                burst_remaining -= 1;
                if burst_remaining == 0 {
                    in_burst = false;
                }
            }
        }
    }
}
