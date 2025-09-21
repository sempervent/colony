#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_udp_simulator_rate() {
        let (tx, mut rx) = mpsc::channel(1000);
        let config = IoSimulatorConfig {
            rate_hz: 10.0,
            jitter_ms: 0,
            burstiness: 0.0,
            loss: 0.0,
            payload_bytes: 100,
            http_paths: vec![],
        };
        
        let simulator = UdpSimulator::new(config);
        let handle = tokio::spawn(async move {
            simulator.run(tx, 42).await;
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
        
        // Should be approximately 10 packets (within reasonable tolerance)
        assert!(packet_count >= 8 && packet_count <= 12, "Expected ~10 packets, got {}", packet_count);
    }

    #[tokio::test]
    async fn test_udp_simulator_loss() {
        let (tx, mut rx) = mpsc::channel(1000);
        let config = IoSimulatorConfig {
            rate_hz: 100.0,
            jitter_ms: 0,
            burstiness: 0.0,
            loss: 0.5, // 50% loss
            payload_bytes: 100,
            http_paths: vec![],
        };
        
        let simulator = UdpSimulator::new(config);
        let handle = tokio::spawn(async move {
            simulator.run(tx, 123).await;
        });
        
        // Collect packets for 1 second
        let start = std::time::Instant::now();
        let mut packet_count = 0;
        
        while start.elapsed() < Duration::from_secs(1) {
            if let Ok(_) = timeout(Duration::from_millis(10), rx.recv()).await {
                packet_count += 1;
            }
        }
        
        handle.abort();
        
        // Should be approximately 50 packets (50% of 100)
        assert!(packet_count >= 40 && packet_count <= 60, "Expected ~50 packets with 50% loss, got {}", packet_count);
    }

    #[tokio::test]
    async fn test_http_parser() {
        let (packet_tx, packet_rx) = mpsc::channel(100);
        let (ops_tx, mut ops_rx) = mpsc::channel(100);
        
        let parser = HttpParser::new();
        let handle = tokio::spawn(async move {
            parser.start(packet_rx, ops_tx).await;
        });
        
        // Send HTTP request
        let request = IoPacket::HttpReq {
            ts_ns: 123456789,
            path: "/api/test".to_string(),
            headers: vec![("Content-Type".to_string(), "application/json".to_string())],
            body: bytes::Bytes::from("{\"test\": true}"),
        };
        
        packet_tx.send(request).await.unwrap();
        
        // Should receive parsed op
        let parsed_op = timeout(Duration::from_millis(100), ops_rx.recv()).await.unwrap().unwrap();
        match parsed_op {
            ParsedOp::HttpMessage { is_req, bytes } => {
                assert!(is_req);
                assert_eq!(bytes, bytes::Bytes::from("{\"test\": true}"));
            }
            _ => panic!("Expected HttpMessage"),
        }
        
        handle.abort();
    }
}
