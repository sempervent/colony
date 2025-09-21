use super::{IoPacket, ParsedOp, IoParser};
use bytes::Bytes;
use tokio::sync::mpsc;

pub struct HttpParser;

impl HttpParser {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl IoParser for HttpParser {
    async fn start(self: Box<Self>, mut rx: mpsc::Receiver<IoPacket>, tx_ops: mpsc::Sender<ParsedOp>) {
        while let Some(packet) = rx.recv().await {
            match packet {
                IoPacket::HttpReq { body, .. } => {
                    let parsed = ParsedOp::HttpMessage {
                        is_req: true,
                        bytes: body,
                    };
                    if tx_ops.send(parsed).await.is_err() {
                        break;
                    }
                }
                IoPacket::HttpResp { body, .. } => {
                    let parsed = ParsedOp::HttpMessage {
                        is_req: false,
                        bytes: body,
                    };
                    if tx_ops.send(parsed).await.is_err() {
                        break;
                    }
                }
                IoPacket::Udp { .. } => {
                    // Ignore UDP packets in HTTP parser
                }
            }
        }
    }
}
