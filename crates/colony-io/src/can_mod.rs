use serde::{Serialize, Deserialize};
use tokio::sync::mpsc;
use bytes::Bytes;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use tokio::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanSimConfig {
    pub rate_hz: f32,
    pub jitter_ms: u16,
    pub burstiness: f32,
    pub error_rate: f32,    // frame errors/arbitration loss
    pub id_space: (u32, u32),
}

impl Default for CanSimConfig {
    fn default() -> Self {
        Self {
            rate_hz: 50.0,
            jitter_ms: 2,
            burstiness: 0.1,
            error_rate: 0.01,
            id_space: (0x100, 0x7FF),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModbusSimConfig {
    pub rate_hz: f32,
    pub loss: f32,
    pub jitter_ms: u16,
    pub fcodes: Vec<u8>,    // supported function codes
    pub payload_bytes: usize,
}

impl Default for ModbusSimConfig {
    fn default() -> Self {
        Self {
            rate_hz: 10.0,
            loss: 0.02,
            jitter_ms: 5,
            fcodes: vec![0x03, 0x04, 0x06, 0x10], // Read Holding, Read Input, Write Single, Write Multiple
            payload_bytes: 256,
        }
    }
}

#[derive(Debug, Clone)]
pub enum CanPacket { 
    Data { id: u32, dlc: u8, bytes: [u8; 8] }, 
    Error 
}

#[derive(Debug, Clone)]
pub enum ModbusPdu { 
    Request { fcode: u8, addr: u16, len: u16 }, 
    Response { fcode: u8, bytes: Bytes } 
}

pub async fn run_can_sim(tx: mpsc::Sender<CanPacket>, cfg: CanSimConfig, seed: u64) {
    let mut rng = Pcg64::seed_from_u64(seed);
    let mut last_packet = Instant::now();
    let mut in_burst = false;
    let mut burst_remaining = 0;
    
    // Poisson inter-arrival time
    let mean_interval_ms = 1000.0 / cfg.rate_hz;
    
    loop {
        // Check if we should start a burst
        if !in_burst && rng.gen::<f32>() < cfg.burstiness {
            in_burst = true;
            burst_remaining = rng.gen_range(2..=6); // 2-6 packets in burst
        }
        
        // Calculate next packet time
        let interval_ms = if in_burst {
            // Shorter intervals during burst
            mean_interval_ms * 0.2
        } else {
            // Poisson inter-arrival
            -rng.gen::<f32>().ln() * mean_interval_ms
        };
        
        let jitter_ms = rng.gen_range(0..=cfg.jitter_ms) as f32;
        let total_delay = interval_ms + jitter_ms;
        
        tokio::time::sleep(Duration::from_millis(total_delay as u64)).await;
        
        // Simulate arbitration errors
        if rng.gen::<f32>() < cfg.error_rate {
            if tx.send(CanPacket::Error).await.is_err() {
                break;
            }
            continue;
        }
        
        // Generate CAN data packet
        let id = rng.gen_range(cfg.id_space.0..=cfg.id_space.1);
        let dlc = rng.gen_range(0..=8);
        let mut bytes = [0u8; 8];
        for i in 0..dlc {
            bytes[i as usize] = rng.gen();
        }
        
        let packet = CanPacket::Data { id, dlc, bytes };
        
        if tx.send(packet).await.is_err() {
            break;
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

pub async fn run_modbus_sim(tx: mpsc::Sender<ModbusPdu>, cfg: ModbusSimConfig, seed: u64) {
    let mut rng = Pcg64::seed_from_u64(seed);
    let mean_interval_ms = 1000.0 / cfg.rate_hz;
    
    loop {
        // Calculate next request time
        let interval_ms = -rng.gen::<f32>().ln() * mean_interval_ms;
        let jitter_ms = rng.gen_range(0..=cfg.jitter_ms) as f32;
        let total_delay = interval_ms + jitter_ms;
        
        tokio::time::sleep(Duration::from_millis(total_delay as u64)).await;
        
        // Simulate packet loss
        if rng.gen::<f32>() < cfg.loss {
            continue;
        }
        
        // Select random function code
        let fcode = cfg.fcodes[rng.gen_range(0..cfg.fcodes.len())];
        let addr = rng.gen_range(0..=65535);
        let len = rng.gen_range(1..=125); // Modbus limit
        
        // Send request
        let request = ModbusPdu::Request { fcode, addr, len };
        if tx.send(request).await.is_err() {
            break;
        }
        
        // Simulate response after a short delay
        tokio::time::sleep(Duration::from_millis(rng.gen_range(10..=50))).await;
        
        // Generate response data
        let mut response_bytes = Vec::with_capacity(cfg.payload_bytes);
        for _ in 0..cfg.payload_bytes {
            response_bytes.push(rng.gen());
        }
        
        let response = ModbusPdu::Response { 
            fcode, 
            bytes: Bytes::from(response_bytes) 
        };
        
        if tx.send(response).await.is_err() {
            break;
        }
    }
}

// Optional real backends (feature-gated):
#[cfg(feature="can_real")]
pub async fn run_can_real(iface: &str, tx: mpsc::Sender<CanPacket>) {
    // TODO: Implement SocketCAN integration
    // This would use the socketcan crate to read from real CAN interfaces
    println!("Real CAN backend not yet implemented for interface: {}", iface);
}

#[cfg(feature="modbus_real")]
pub async fn run_modbus_real_tcp(addr: &str, tx: mpsc::Sender<ModbusPdu>) {
    // TODO: Implement real Modbus TCP client
    // This would use tokio-modbus or similar to connect to real devices
    println!("Real Modbus TCP backend not yet implemented for address: {}", addr);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_can_simulator_rate() {
        let (tx, mut rx) = mpsc::channel(100);
        let config = CanSimConfig {
            rate_hz: 10.0,
            jitter_ms: 0,
            burstiness: 0.0,
            error_rate: 0.0,
            id_space: (0x100, 0x200),
        };
        
        let handle = tokio::spawn(async move {
            run_can_sim(tx, config, 42).await;
        });
        
        // Collect packets for 1 second
        let start = std::time::Instant::now();
        let mut packet_count = 0;
        
        while start.elapsed() < Duration::from_secs(1) {
            if let Ok(_) = timeout(Duration::from_millis(100), rx.recv()).await {
                packet_count += 1;
            }
        }
        
        handle.abort();
        
        // Should be approximately 10 packets
        assert!(packet_count >= 8 && packet_count <= 12, "Expected ~10 packets, got {}", packet_count);
    }

    #[tokio::test]
    async fn test_modbus_simulator() {
        let (tx, mut rx) = mpsc::channel(100);
        let config = ModbusSimConfig {
            rate_hz: 5.0,
            loss: 0.0,
            jitter_ms: 0,
            fcodes: vec![0x03, 0x04],
            payload_bytes: 128,
        };
        
        let handle = tokio::spawn(async move {
            run_modbus_sim(tx, config, 123).await;
        });
        
        // Collect a few request/response pairs
        let mut request_count = 0;
        let mut response_count = 0;
        
        for _ in 0..6 { // Should get 3 pairs
            if let Ok(pdu) = timeout(Duration::from_millis(500), rx.recv()).await {
                match pdu {
                    ModbusPdu::Request { .. } => request_count += 1,
                    ModbusPdu::Response { .. } => response_count += 1,
                }
            }
        }
        
        handle.abort();
        
        assert!(request_count >= 2);
        assert!(response_count >= 2);
    }
}
