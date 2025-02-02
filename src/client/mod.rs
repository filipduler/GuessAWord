use std::net::SocketAddr;
use std::time::Duration;
use tokio_util::bytes::Bytes;
use anyhow::bail;
use futures::{SinkExt, StreamExt, TryStreamExt};
use tokio::net::{TcpStream};
use tokio::time::timeout;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use crate::bin_reader::BinReader;
use crate::bin_writer::{BinWriter};
use crate::utils::{create_framed_stream, ClientId, MAX_PACKET_LENGTH};

mod message;
pub use message::ClientMessage;
use crate::server::ServerMessage;

pub struct Client {
    pub id: ClientId,
    stream: Framed<TcpStream, LengthDelimitedCodec>,
    bin_writer: BinWriter
}

#[derive(Debug, Clone, PartialEq)]
pub enum StreamedMessage {
    Challenged,
    Hint(String),
    Attempt(bool, String),
}

impl Client {
    pub async fn connect_async(addr: SocketAddr, password: &str) -> anyhow::Result<Client> {
        let stream = TcpStream::connect(addr).await?;
        let mut framed_stream = create_framed_stream(stream);

        // expect request authorization message from the server
        let result = framed_stream.next().await;
        let mut reader = BinReader::from_result(&result)?;

        let message: ServerMessage = ServerMessage::try_from(reader.read_u8())?;
        if message != ServerMessage::RequestAuthorization {
            bail!("Unexpected server message '{:?}'", message);
        }

        // send password
        let mut bin_writer = BinWriter::with_capacity(MAX_PACKET_LENGTH);
        bin_writer.write_u8(ClientMessage::Authorization as u8);
        bin_writer.write_str(password);

        framed_stream.send(Bytes::from(bin_writer.clone_data())).await?;

        // expect client id or error
        let result = framed_stream.next().await;
        let mut reader = BinReader::from_result(&result)?;

        Ok(Client{
            id: reader.read_u32(),
            stream: framed_stream,
            bin_writer
        })
    }


    pub async fn get_opponents_async(&mut self) -> anyhow::Result<Vec<u32>> {
        self.bin_writer.clear();

        // send request
        self.bin_writer.write_u8(ClientMessage::ListOfOpponents as u8);
        self.stream.send(Bytes::from(self.bin_writer.clone_data())).await?;

        // receive response
        let result = self.stream.next().await;
        let mut reader = BinReader::from_result(&result)?;

        let message = ServerMessage::try_from(reader.read_u8())?;
        Ok(match message {
            ServerMessage::ListOfOpponents => {
                let opponents_count = reader.read_u16() as usize;
                let mut opponents = Vec::with_capacity(opponents_count);

                for _ in 0..opponents_count {
                    opponents.push(reader.read_u32());
                }

                opponents
            },
            _ => bail!("Unexpected server message '{:?}'", message)
        })
    }

    pub async fn request_match_async(&mut self, opponent_id: ClientId, word: &str) -> anyhow::Result<()> {
        self.bin_writer.clear();

        // send request
        self.bin_writer.write_u8(ClientMessage::RequestMatch as u8);
        self.bin_writer.write_u32(opponent_id);
        self.bin_writer.write_str(word);
        self.stream.send(Bytes::from(self.bin_writer.clone_data())).await?;

        // receive response
        let result = self.stream.next().await;
        let mut reader = BinReader::from_result(&result)?;

        let message = ServerMessage::try_from(reader.read_u8())?;
        match message {
            ServerMessage::Ok => {},
            ServerMessage::Err => bail!("Couldn't begin match with opponent"),
            _ => bail!("Unexpected server message '{:?}'", message)
        }

        Ok(())
    }

    pub async fn send_hint_async(&mut self, hint: &str) -> anyhow::Result<()> {
        self.bin_writer.clear();

        // send request
        self.bin_writer.write_u8(ClientMessage::SendHint as u8);
        self.bin_writer.write_str(hint);
        self.stream.send(Bytes::from(self.bin_writer.clone_data())).await?;

        Ok(())
    }

    pub async fn send_attempt_async(&mut self, word: &str) -> anyhow::Result<bool> {
        self.bin_writer.clear();

        // send request
        self.bin_writer.write_u8(ClientMessage::SendAttempt as u8);
        self.bin_writer.write_str(word);
        self.stream.send(Bytes::from(self.bin_writer.clone_data())).await?;

        // receive response
        let result = self.stream.next().await;
        let mut reader = BinReader::from_result(&result)?;

        let message = ServerMessage::try_from(reader.read_u8())?;
        match message {
            ServerMessage::AttemptResult => {
                Ok(reader.read_u8() > 0)
            },
            _ => bail!("Unexpected server message '{:?}'", message)
        }
    }

    pub async fn read_streamed_message_async(&mut self) -> anyhow::Result<Option<StreamedMessage>> {
        let duration = Duration::from_millis(50);

        match timeout(duration, self.stream.next()).await {
            Ok(result) => {
                let mut reader = BinReader::from_result(&result)?;

                let message = ServerMessage::try_from(reader.read_u8())?;
                return Ok(match message {
                    ServerMessage::Challenged => Some(StreamedMessage::Challenged),
                    ServerMessage::Attempt => Some(StreamedMessage::Attempt(reader.read_u8() > 0, reader.read_str())),
                    ServerMessage::Hint => Some(StreamedMessage::Hint(reader.read_str())),
                    _ => bail!("Unexpected server message '{:?}'", message)
                })
            }
            Err(_) => {
                eprintln!("Read timed out");
            }
        }

        Ok(None)
    }
}