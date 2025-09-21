use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IoPacket {
    Udp(Vec<u8>),
    Can(CanFrame),
    Tcp(bytes::Bytes),
    Http(HttpEvent),
    Modbus(ModbusPDU),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanFrame {
    pub id: u32,
    pub data: Vec<u8>,
    pub extended: bool,
    pub remote: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpEvent {
    Request {
        method: String,
        path: String,
        headers: std::collections::HashMap<String, String>,
        body: Vec<u8>,
    },
    Response {
        status: u16,
        headers: std::collections::HashMap<String, String>,
        body: Vec<u8>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModbusPDU {
    pub function_code: u8,
    pub data: Vec<u8>,
}
