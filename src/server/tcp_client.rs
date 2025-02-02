use futures::channel::mpsc::UnboundedSender;
use futures::SinkExt;
use tokio_util::bytes::Bytes;
use crate::bin_writer::BinWriter;
use crate::server::TcpMessage;
use crate::utils::ClientId;

pub struct TcpClient {
    pub id: ClientId,
    sender: UnboundedSender<TcpMessage>
}

impl TcpClient {
    pub fn new(id: ClientId, sender: UnboundedSender<TcpMessage>) -> TcpClient {
        TcpClient {
            id,
            sender
        }
    }

    pub async fn disconnect_async(&mut self) -> anyhow::Result<()> {
        self
            .sender
            .send(TcpMessage::Disconnect(self.id))
            .await?;

        Ok(())
    }

    pub async fn send_async(&mut self, bin_writer: &BinWriter) -> anyhow::Result<()> {
        if bin_writer.len() > 0 {
            self
                .sender
                .send(TcpMessage::Message(
                    self.id,
                    Bytes::from(bin_writer.clone_data()),
                ))
                .await?;
        }

        Ok(())
    }
}