use serde::{Deserialize, Serialize};
use colony_core::{Op, QoS};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModContent {
    pub pipelines: Vec<PipelineDef>,
    pub events: Vec<BlackSwanEvent>,
    pub tech: Vec<TechDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineDef {
    pub id: String,
    pub ops: Vec<String>,
    pub qos: String,
    pub deadline_ms: u64,
    pub payload_sz: usize,
}

impl PipelineDef {
    pub fn to_pipeline(&self) -> Result<colony_core::Pipeline, String> {
        let ops: Result<Vec<Op>, _> = self.ops
            .iter()
            .map(|op_str| match op_str.as_str() {
                "Decode" => Ok(Op::Decode),
                "Fft" => Ok(Op::Fft),
                "Kalman" => Ok(Op::Kalman),
                "Yolo" => Ok(Op::Yolo),
                "Crc" => Ok(Op::Crc),
                "CanParse" => Ok(Op::CanParse),
                "UdpDemux" => Ok(Op::UdpDemux),
                "TcpSessionize" => Ok(Op::TcpSessionize),
                "ModbusMap" => Ok(Op::ModbusMap),
                "HttpParse" => Ok(Op::HttpParse),
                _ => Err(format!("Unknown operation: {}", op_str)),
            })
            .collect();

        let ops = ops?;

        let qos = match self.qos.as_str() {
            "Throughput" => QoS::Throughput,
            "Latency" => QoS::Latency,
            "Balanced" => QoS::Balanced,
            _ => return Err(format!("Unknown QoS: {}", self.qos)),
        };

        Ok(colony_core::Pipeline {
            ops,
            mutation_tag: None,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlackSwanEvent {
    pub id: String,
    pub name: String,
    pub triggers: Vec<String>,
    pub effects: Vec<String>,
    pub cure: Option<String>,
    pub weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cost: u32,
    pub prerequisites: Vec<String>,
    pub effects: Vec<String>,
}
