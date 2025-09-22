pub mod parsers;
pub mod simulators;
// pub mod packets; // Removed - was causing conflicts
pub mod udp_sim;
pub mod http_sim;
pub mod http_parse;
pub mod can_mod;

#[cfg(test)]
mod tests;

pub use parsers::*;
pub use simulators::*;
// pub use packets::*; // Removed - was causing conflicts
pub use udp_sim::UdpSimulator;
pub use http_sim::HttpSimulator;
pub use http_parse::HttpParser;
pub use can_mod::{CanSimConfig, ModbusSimConfig, CanPacket, ModbusPdu, run_can_sim, run_modbus_sim};

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoSimulatorConfig {
    pub rate_hz: f32,      // mean packets per second
    pub jitter_ms: u16,    // scheduling jitter
    pub burstiness: f32,   // 0..1, how clumpy
    pub loss: f32,         // 0..1
    pub payload_bytes: usize,
    pub http_paths: Vec<String>, // for HTTP sim
}

impl Default for IoSimulatorConfig {
    fn default() -> Self {
        Self {
            rate_hz: 100.0,
            jitter_ms: 5,
            burstiness: 0.1,
            loss: 0.01,
            payload_bytes: 1024,
            http_paths: vec!["/api/metrics".to_string(), "/api/status".to_string()],
        }
    }
}

#[derive(Debug, Clone)]
pub enum IoPacket {
    Udp { ts_ns: u64, src: std::net::SocketAddr, data: Bytes },
    HttpReq { ts_ns: u64, path: String, headers: Vec<(String, String)>, body: Bytes },
    HttpResp { ts_ns: u64, code: u16, headers: Vec<(String, String)>, body: Bytes },
}

// Output to the ECS op executor
#[derive(Debug, Clone)]
pub enum ParsedOp {
    UdpFrame { payload: Bytes },
    HttpMessage { is_req: bool, bytes: Bytes },
}

#[async_trait::async_trait]
pub trait IoSource: Send + Sync {
    async fn run(self: Box<Self>, tx: mpsc::Sender<IoPacket>, seed: u64);
}

#[async_trait::async_trait]
pub trait IoParser: Send + Sync {
    async fn start(self: Box<Self>, rx: mpsc::Receiver<IoPacket>, tx_ops: mpsc::Sender<ParsedOp>);
}

pub async fn run_udp_sim(
    tx: mpsc::Sender<IoPacket>,
    cfg: IoSimulatorConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(
        (1000.0 / cfg.rate_hz) as u64,
    ));

    loop {
        interval.tick().await;
        
        // Simulate packet loss
        if rand::random::<f32>() < cfg.loss {
            continue;
        }

        // Create a simulated UDP packet
        let packet = IoPacket::Udp {
            ts_ns: chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64,
            src: "127.0.0.1:1234".parse().unwrap(),
            data: Bytes::from(vec![
                0x45, 0x00, 0x00, 0x20, // IP header
                0x00, 0x01, 0x00, 0x00, // More IP header
                0x40, 0x11, 0x00, 0x00, // UDP protocol
                0x7f, 0x00, 0x00, 0x01, // Source IP
                0x7f, 0x00, 0x00, 0x01, // Dest IP
                0x12, 0x34, 0x56, 0x78, // Source/Dest ports
                0x00, 0x0c, 0x00, 0x00, // Length/Checksum
                0x48, 0x65, 0x6c, 0x6c, // "Hell"
                0x6f, 0x20, 0x57, 0x6f, // "o Wo"
                0x72, 0x6c, 0x64, 0x21, // "rld!"
            ]),
        };

        // Simulate jitter
        if cfg.jitter_ms > 0 {
            let jitter = rand::random::<u16>() % cfg.jitter_ms;
            tokio::time::sleep(tokio::time::Duration::from_millis(jitter as u64)).await;
        }

        if tx.send(packet).await.is_err() {
            break;
        }
    }

    Ok(())
}

pub async fn parse_udp(
    rx: &mut tokio::net::UdpSocket,
    tx: mpsc::Sender<IoPacket>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut buf = [0u8; 1024];
    
    loop {
        let (len, _addr) = rx.recv_from(&mut buf).await?;
        let packet = IoPacket::Udp {
            ts_ns: chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64,
            src: "127.0.0.1:1234".parse().unwrap(),
            data: Bytes::from(buf[..len].to_vec()),
        };
        
        if tx.send(packet).await.is_err() {
            break Ok(());
        }
    }
}
