use bevy::prelude::*;
use colony_io::{IoSimulatorConfig, UdpSimulator, HttpSimulator, HttpParser, IoPacket, ParsedOp, IoSource, IoParser};
use tokio::sync::mpsc;
use super::{Job, QoS};

#[derive(Resource, Clone)]
pub struct IoRuntime {
    pub udp_tx: Option<mpsc::Sender<IoPacket>>,
    pub http_tx: Option<mpsc::Sender<IoPacket>>,
    pub job_tx: Option<mpsc::Sender<Job>>,
}

impl Default for IoRuntime {
    fn default() -> Self {
        Self {
            udp_tx: None,
            http_tx: None,
            job_tx: None,
        }
    }
}

pub async fn start_io_runtime(
    seed: u64, 
    udp_cfg: IoSimulatorConfig, 
    http_cfg: IoSimulatorConfig,
    job_tx: mpsc::Sender<Job>
) {
    // Create channels
    let (udp_packet_tx, udp_packet_rx) = mpsc::channel(1000);
    let (http_packet_tx, http_packet_rx) = mpsc::channel(1000);
    let (udp_ops_tx, mut udp_ops_rx) = mpsc::channel(1000);
    let (http_ops_tx, mut http_ops_rx) = mpsc::channel(1000);
    
    // Start UDP simulator
    let udp_sim = UdpSimulator::new(udp_cfg);
    tokio::spawn(async move {
        Box::new(udp_sim).run(udp_packet_tx, seed).await;
    });
    
    // Start HTTP simulator
    let http_sim = HttpSimulator::new(http_cfg);
    tokio::spawn(async move {
        Box::new(http_sim).run(http_packet_tx, seed + 1).await;
    });
    
    // Start UDP framer (simple passthrough for now)
    tokio::spawn(async move {
        let mut rx = udp_packet_rx;
        while let Some(packet) = rx.recv().await {
            if let IoPacket::Udp { data, .. } = packet {
                let parsed = ParsedOp::UdpFrame { payload: data };
                if udp_ops_tx.send(parsed).await.is_err() {
                    break;
                }
            }
        }
    });
    
    // Start HTTP parser
    let http_parser = HttpParser::new();
    tokio::spawn(async move {
        Box::new(http_parser).start(http_packet_rx, http_ops_tx).await;
    });
    
    // Job enqueuer for UDP
    let job_tx_udp = job_tx.clone();
    tokio::spawn(async move {
        while let Some(parsed_op) = udp_ops_rx.recv().await {
            if let ParsedOp::UdpFrame { payload } = parsed_op {
                enqueue_job_for_pipeline("udp_telemetry_ingest", payload.len(), &job_tx_udp).await;
            }
        }
    });
    
    // Job enqueuer for HTTP
    tokio::spawn(async move {
        while let Some(parsed_op) = http_ops_rx.recv().await {
            if let ParsedOp::HttpMessage { bytes, .. } = parsed_op {
                enqueue_job_for_pipeline("http_ingest", bytes.len(), &job_tx).await;
            }
        }
    });
}

async fn enqueue_job_for_pipeline(pipeline_id: &str, payload_sz: usize, job_tx: &mpsc::Sender<Job>) {
    if let Some(pipeline) = super::pipelines::get_pipeline_by_id(pipeline_id) {
        let job = Job {
            id: chrono::Utc::now().timestamp_millis() as u64,
            pipeline,
            qos: match pipeline_id {
                "udp_telemetry_ingest" => QoS::Balanced,
                "http_ingest" => QoS::Latency,
                _ => QoS::Balanced,
            },
            deadline_ms: match pipeline_id {
                "udp_telemetry_ingest" => 50,
                "http_ingest" => 100,
                _ => 100,
            },
            payload_sz,
        };
        
        let _ = job_tx.send(job).await;
    }
}
