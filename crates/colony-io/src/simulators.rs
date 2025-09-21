use crate::{IoPacket, IoSimulatorConfig};
use tokio::sync::mpsc;

pub struct UdpSimulator {
    config: IoSimulatorConfig,
}

impl UdpSimulator {
    pub fn new(config: IoSimulatorConfig) -> Self {
        Self { config }
    }

    pub async fn run(&self, tx: mpsc::Sender<IoPacket>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(
            (1000.0 / self.config.rate_hz) as u64,
        ));

        loop {
            interval.tick().await;
            
            // Simulate packet loss
            if rand::random::<f32>() < self.config.loss {
                continue;
            }

            // Create a simulated UDP packet with telemetry data
            let telemetry_data = format!(
                "{{\"timestamp\":{},\"cpu_usage\":{:.2},\"memory_usage\":{:.2},\"temperature\":{:.1}}}",
                chrono::Utc::now().timestamp_millis(),
                rand::random::<f32>() * 100.0,
                rand::random::<f32>() * 100.0,
                20.0 + rand::random::<f32>() * 60.0
            );

            let packet = IoPacket::Udp(telemetry_data.into_bytes());

            // Simulate jitter
            if self.config.jitter_ms > 0 {
                let jitter = rand::random::<u16>() % self.config.jitter_ms;
                tokio::time::sleep(tokio::time::Duration::from_millis(jitter as u64)).await;
            }

            if tx.send(packet).await.is_err() {
                break;
            }
        }

        Ok(())
    }
}

pub struct TcpSimulator {
    config: IoSimulatorConfig,
}

impl TcpSimulator {
    pub fn new(config: IoSimulatorConfig) -> Self {
        Self { config }
    }

    pub async fn run(&self, tx: mpsc::Sender<IoPacket>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(
            (1000.0 / self.config.rate_hz) as u64,
        ));

        loop {
            interval.tick().await;
            
            // Simulate packet loss
            if rand::random::<f32>() < self.config.loss {
                continue;
            }

            // Create simulated HTTP request
            let http_request = format!(
                "GET /api/metrics HTTP/1.1\r\n\
                Host: localhost:8080\r\n\
                User-Agent: Colony-Simulator/1.0\r\n\
                Accept: application/json\r\n\
                \r\n"
            );

            let packet = IoPacket::Tcp(bytes::Bytes::from(http_request));

            // Simulate jitter
            if self.config.jitter_ms > 0 {
                let jitter = rand::random::<u16>() % self.config.jitter_ms;
                tokio::time::sleep(tokio::time::Duration::from_millis(jitter as u64)).await;
            }

            if tx.send(packet).await.is_err() {
                break;
            }
        }

        Ok(())
    }
}
