use crate::packets::*;
use bytes::Bytes;

pub struct UdpParser;

impl UdpParser {
    pub fn parse(&self, data: &[u8]) -> Result<IoPacket, ParserError> {
        if data.len() < 8 {
            return Err(ParserError::InsufficientData);
        }
        
        // Basic UDP header parsing
        let src_port = u16::from_be_bytes([data[0], data[1]]);
        let dst_port = u16::from_be_bytes([data[2], data[3]]);
        let length = u16::from_be_bytes([data[4], data[5]]);
        let checksum = u16::from_be_bytes([data[6], data[7]]);
        
        if data.len() < length as usize {
            return Err(ParserError::InsufficientData);
        }
        
        let payload = data[8..length as usize].to_vec();
        
        Ok(IoPacket::Udp(payload))
    }
}

pub struct TcpParser;

impl TcpParser {
    pub fn parse(&self, data: &[u8]) -> Result<IoPacket, ParserError> {
        if data.len() < 20 {
            return Err(ParserError::InsufficientData);
        }
        
        // Basic TCP header parsing
        let src_port = u16::from_be_bytes([data[0], data[1]]);
        let dst_port = u16::from_be_bytes([data[2], data[3]]);
        let seq_num = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let ack_num = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
        
        let data_offset = (data[12] >> 4) as usize;
        if data.len() < data_offset * 4 {
            return Err(ParserError::InsufficientData);
        }
        
        let payload = Bytes::copy_from_slice(&data[data_offset * 4..]);
        
        Ok(IoPacket::Tcp(payload))
    }
}

pub struct HttpParser;

impl HttpParser {
    pub fn parse(&self, data: &[u8]) -> Result<IoPacket, ParserError> {
        let text = String::from_utf8_lossy(data);
        
        if text.starts_with("HTTP/") {
            // Response
            let lines: Vec<&str> = text.lines().collect();
            if lines.is_empty() {
                return Err(ParserError::InvalidFormat);
            }
            
            let status_line = lines[0];
            let status = status_line
                .split_whitespace()
                .nth(1)
                .and_then(|s| s.parse::<u16>().ok())
                .unwrap_or(200);
            
            let mut headers = std::collections::HashMap::new();
            let mut body_start = 0;
            
            for (i, line) in lines.iter().enumerate() {
                if line.is_empty() {
                    body_start = i + 1;
                    break;
                }
                if let Some((key, value)) = line.split_once(':') {
                    headers.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
            
            let body = if body_start < lines.len() {
                lines[body_start..].join("\n").into_bytes()
            } else {
                Vec::new()
            };
            
            Ok(IoPacket::Http(HttpEvent::Response {
                status,
                headers,
                body,
            }))
        } else {
            // Request
            let lines: Vec<&str> = text.lines().collect();
            if lines.is_empty() {
                return Err(ParserError::InvalidFormat);
            }
            
            let request_line = lines[0];
            let parts: Vec<&str> = request_line.split_whitespace().collect();
            if parts.len() < 3 {
                return Err(ParserError::InvalidFormat);
            }
            
            let method = parts[0].to_string();
            let path = parts[1].to_string();
            
            let mut headers = std::collections::HashMap::new();
            let mut body_start = 0;
            
            for (i, line) in lines.iter().enumerate() {
                if line.is_empty() {
                    body_start = i + 1;
                    break;
                }
                if let Some((key, value)) = line.split_once(':') {
                    headers.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
            
            let body = if body_start < lines.len() {
                lines[body_start..].join("\n").into_bytes()
            } else {
                Vec::new()
            };
            
            Ok(IoPacket::Http(HttpEvent::Request {
                method,
                path,
                headers,
                body,
            }))
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParserError {
    #[error("Insufficient data")]
    InsufficientData,
    #[error("Invalid format")]
    InvalidFormat,
    #[error("Checksum mismatch")]
    ChecksumMismatch,
}
