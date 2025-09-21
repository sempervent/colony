use super::{IoPacket, IoSimulatorConfig, IoSource};
use bytes::Bytes;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use tokio::sync::mpsc;
use tokio::time::Duration;

pub struct HttpSimulator {
    config: IoSimulatorConfig,
}

impl HttpSimulator {
    pub fn new(config: IoSimulatorConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl IoSource for HttpSimulator {
    async fn run(self: Box<Self>, tx: mpsc::Sender<IoPacket>, seed: u64) {
        let mut rng = StdRng::seed_from_u64(seed);
        let mean_interval_ms = 1000.0 / self.config.rate_hz;
        
        loop {
            // Calculate next request time
            let interval_ms = -rng.gen::<f32>().ln() * mean_interval_ms;
            let jitter_ms = rng.gen_range(0..=self.config.jitter_ms) as f32;
            let total_delay = interval_ms + jitter_ms;
            
            tokio::time::sleep(Duration::from_millis(total_delay as u64)).await;
            
            // Simulate packet loss
            if rng.gen::<f32>() < self.config.loss {
                continue;
            }
            
            let now = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64;
            
            // Select random path
            let path = self.config.http_paths
                .get(rng.gen_range(0..self.config.http_paths.len()))
                .cloned()
                .unwrap_or_else(|| "/api/default".to_string());
            
            // Generate HTTP request
            let request_body = format!(
                r#"{{"timestamp":{},"query":"test","params":{{"limit":{},"offset":{}}}}}"#,
                now,
                rng.gen_range(1..=100),
                rng.gen_range(0..=1000),
            );
            
            let request_headers = vec![
                ("Content-Type".to_string(), "application/json".to_string()),
                ("User-Agent".to_string(), "Colony-Simulator/1.0".to_string()),
                ("Accept".to_string(), "application/json".to_string()),
            ];
            
            let req_packet = IoPacket::HttpReq {
                ts_ns: now,
                path: path.clone(),
                headers: request_headers,
                body: Bytes::from(request_body),
            };
            
            if tx.send(req_packet).await.is_err() {
                break;
            }
            
            // Simulate response after a short delay
            tokio::time::sleep(Duration::from_millis(rng.gen_range(10..=50))).await;
            
            let response_body = format!(
                r#"{{"status":"ok","data":{{"count":{},"results":[]}},"timestamp":{}}}"#,
                rng.gen_range(0..=100),
                now + 1000000, // 1ms later
            );
            
            let response_headers = vec![
                ("Content-Type".to_string(), "application/json".to_string()),
                ("Content-Length".to_string(), response_body.len().to_string()),
                ("Server".to_string(), "Colony-API/1.0".to_string()),
            ];
            
            let resp_packet = IoPacket::HttpResp {
                ts_ns: now + 1000000,
                path,
                code: 200,
                headers: response_headers,
                body: Bytes::from(response_body),
            };
            
            if tx.send(resp_packet).await.is_err() {
                break;
            }
        }
    }
}
