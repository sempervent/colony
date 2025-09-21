use serde::{Deserialize, Serialize};
use super::{Op, Pipeline};

#[derive(Serialize, Deserialize, Clone)]
pub struct PipelineDef {
    pub id: String,
    pub ops: Vec<String>,
    pub qos: String,
    pub deadline_ms: u64,
    pub payload_sz: usize,
}

impl PipelineDef {
    pub fn to_pipeline(&self) -> Result<Pipeline, String> {
        let ops: Result<Vec<Op>, _> = self.ops
            .iter()
            .map(|op_str| match op_str.as_str() {
                "UdpDemux" => Ok(Op::UdpDemux),
                "Decode" => Ok(Op::Decode),
                "Kalman" => Ok(Op::Kalman),
                "Export" => Ok(Op::Export),
                "HttpParse" => Ok(Op::HttpParse),
                "HttpExport" => Ok(Op::HttpExport),
                "Fft" => Ok(Op::Fft),
                "Yolo" => Ok(Op::Yolo),
                "Crc" => Ok(Op::Crc),
                "CanParse" => Ok(Op::CanParse),
                "TcpSessionize" => Ok(Op::TcpSessionize),
                "ModbusMap" => Ok(Op::ModbusMap),
                "MaintenanceCool" => Ok(Op::MaintenanceCool),
                _ => Err(format!("Unknown operation: {}", op_str)),
            })
            .collect();

        let ops = ops?;

        Ok(Pipeline {
            ops,
            mutation_tag: None,
        })
    }
}

pub fn builtin_pipelines() -> Vec<Pipeline> {
    vec![
        Pipeline { 
            ops: vec![Op::UdpDemux, Op::Decode, Op::Kalman, Op::Export], 
            mutation_tag: None 
        },
        Pipeline { 
            ops: vec![Op::HttpParse, Op::HttpExport], 
            mutation_tag: None 
        },
    ]
}

pub fn get_pipeline_by_id(id: &str) -> Option<Pipeline> {
    match id {
        "udp_telemetry_ingest" => Some(Pipeline {
            ops: vec![Op::UdpDemux, Op::Decode, Op::Kalman, Op::Export],
            mutation_tag: None,
        }),
        "http_ingest" => Some(Pipeline {
            ops: vec![Op::HttpParse, Op::HttpExport],
            mutation_tag: None,
        }),
        "can_telemetry" => Some(Pipeline {
            ops: vec![Op::Decode, Op::Kalman, Op::GpuPreprocess, Op::Yolo, Op::GpuExport],
            mutation_tag: None,
        }),
        "modbus_poll" => Some(Pipeline {
            ops: vec![Op::Decode, Op::Kalman, Op::Export],
            mutation_tag: None,
        }),
        _ => None,
    }
}
