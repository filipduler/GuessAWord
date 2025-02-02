use crate::bin_reader::BinReader;
use crate::bin_writer::BinWriter;
use crate::utils::{create_framed_stream, ClientId, MAX_PACKET_LENGTH};
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use anyhow::bail;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio_util::bytes::Bytes;

mod game;
mod message;
mod tcp_client;

use crate::server::game::Game;
use crate::server::tcp_client::TcpClient;
pub use message::ServerMessage;
use crate::client::ClientMessage;

#[derive(Debug, Clone)]
pub enum TcpMessage {
    Message(u32, Bytes),
    Disconnect(u32),
}

pub async fn run_async<A: ToSocketAddrs>(addr: A, password: &str) -> anyhow::Result<()> {
    let mut client_id_counter: u32 = 1;
    let mut clients: HashMap<u32, TcpClient> = HashMap::new();
    let mut game = Game::new(password);
    let (in_sender, mut in_receiver) = futures::channel::mpsc::unbounded::<TcpMessage>();

    let listener = TcpListener::bind(addr).await?;

    let mut bin_writer = BinWriter::with_capacity(MAX_PACKET_LENGTH);

    loop {
        tokio::select! {
            // accept new TCP connections
            Ok((stream, _)) = listener.accept() => {

                // get new client id
                let new_id = client_id_counter;
                client_id_counter += 1;

                // accept the client
                match accept_client(
                    new_id,
                    stream,
                    in_sender.clone(),
                    &mut clients).await {
                        Ok(()) => game.add_player(new_id),
                        Err(err) => eprintln!("Couldn't accept client: {}", err)
                    }
            },
            Some(msg) = in_receiver.next() => {
                    if let TcpMessage::Message(client_id, ref msg) = msg {

                    let mut bin_reader = BinReader::from_bytes(msg);

                    if let Err(err) = process_message(
                        client_id,
                        &mut game,
                        &mut clients,
                        &mut bin_reader,
                        &mut bin_writer).await {

                        eprintln!("Couldn't process message: {}", err);

                        // on error send Err flag
                        if let Some(client) = clients.get_mut(&client_id) {
                            bin_writer.clear();
                            bin_writer.write_u8(ServerMessage::Err as u8);
                            _ = client.send_async(&bin_writer);
                        }
                    }
                }
                 else if let TcpMessage::Disconnect(client_id) = msg {
                    clients.remove(&client_id);
                    game.remove_player(client_id);
                }
            }
        }
    }
}

async fn process_message(
    client_id: ClientId,
    game: &mut Game,
    clients: &mut HashMap<u32, TcpClient>,
    bin_reader: &mut BinReader<'_>,
    bin_writer: &mut BinWriter,
) -> anyhow::Result<()> {
    bin_writer.clear();
    let mut disconnect = false;

    let flag = ClientMessage::try_from(bin_reader.read_u8())?;
    match flag {
        ClientMessage::Authorization => {
            let password = bin_reader.read_str();

            if game.authorize(&password) {
                bin_writer.write_u32(client_id)
            } else {
                disconnect = true;
            }
        }
        ClientMessage::ListOfOpponents => {
            // get list of clients and filter out the callers id
            let opponent_ids: Vec<ClientId> = game
                .list_of_opponents()
                .into_iter()
                .filter(|id| *id != client_id)
                .collect();

            bin_writer.write_u8(ServerMessage::ListOfOpponents as u8);
            bin_writer.write_u16(opponent_ids.len() as u16);
            for id in opponent_ids {
                bin_writer.write_u32(id as u32);
            }
        },
        ClientMessage::RequestMatch => {
            let opponent_id = bin_reader.read_u32();
            let word = bin_reader.read_str();

            if let Some(opponent) = clients.get_mut(&opponent_id) {

                // begin the message
                game.begin_match(client_id, opponent_id, &word)?;

                // send challenge message to the opponent
                bin_writer.write_u8(ServerMessage::Challenged as u8);
                opponent.send_async(bin_writer).await?;
                bin_writer.clear();

                // respond to the caller
                bin_writer.write_u8(ServerMessage::Ok as u8);
            } else {
                bail!("Opponent with id {} not found", opponent_id);
            }
        },
        ClientMessage::SendHint => {
            let hint = bin_reader.read_str();

            let opponent_id = game.opponents_id(client_id)?;
            if let Some(opponent) = clients.get_mut(&opponent_id) {
                // send hint to opponent
                bin_writer.write_u8(ServerMessage::Hint as u8);
                bin_writer.write_str(&hint);
                opponent.send_async(bin_writer).await?;

                bin_writer.clear();
            }
        },
        ClientMessage::SendAttempt => {
            let word = bin_reader.read_str();

            let is_valid = if game.validate_word(client_id, &word)? { 1_u8 } else { 0 };

            // send attempt to challenger
            let challenger_id = game.challenger_id(client_id)?;
            if let Some(challenger) = clients.get_mut(&challenger_id) {

                // send hint to opponent
                bin_writer.write_u8(ServerMessage::Attempt as u8);
                bin_writer.write_u8(is_valid);
                bin_writer.write_str(&word);
                challenger.send_async(bin_writer).await?;

                bin_writer.clear();
            }

            // send response to opponent
            bin_writer.write_u8(ServerMessage::AttemptResult as u8);
            bin_writer.write_u8(is_valid);
        }
        _ => bail!("Unexpected command '{:?}'", flag)
    }


    // send the response
    match clients.get_mut(&client_id) {
        Some(client) => {
            client.send_async(bin_writer).await?;

            if disconnect {
                client.disconnect_async().await?;
            }
        },
        None => bail!("Couldn't find client with id {}", client_id)
    };

    Ok(())
}

async fn accept_client(
    client_id: ClientId,
    stream: TcpStream,
    mut in_sender: UnboundedSender<TcpMessage>,
    client_list: &mut HashMap<u32, TcpClient>,
) -> anyhow::Result<()> {
    // for events going from the server -> client
    let (out_sender, mut out_receiver) = futures::channel::mpsc::unbounded();

    // insert the client to the server list
    client_list.insert(client_id, TcpClient::new(client_id, out_sender));

    // start processing IO for the newly accepted client
    tokio::spawn(async move {
        if let Err(err) = process_client(client_id, stream, &mut in_sender, &mut out_receiver).await
        {
            eprintln!("Error processing client: {}", err);
        }

        in_sender.send(TcpMessage::Disconnect(client_id)).await
    });

    Ok(())
}

async fn process_client(
    client_id: ClientId,
    stream: TcpStream,
    sender: &mut UnboundedSender<TcpMessage>,
    receiver: &mut UnboundedReceiver<TcpMessage>,
) -> anyhow::Result<()> {
    let mut framed_stream = create_framed_stream(stream);

    // send initial request authorization message
    let mut bin_writer = BinWriter::with_capacity(1);
    bin_writer.write_u8(ServerMessage::RequestAuthorization as u8);
    framed_stream.send(Bytes::from(bin_writer.clone_data())) .await?;

    loop {
        tokio::select! {
            // process outgoing messages - server -> client
            Some(msg_type) = receiver.next() => {
                match msg_type {
                    TcpMessage::Message(_, msg) => {
                        framed_stream.send(msg).await?;
                    },
                    TcpMessage::Disconnect(_) => {
                        framed_stream.close().await?;
                    },
                }
            },
            // process incoming message - client -> server
        result = framed_stream.next() => match result {
            Some(Ok(msg)) => {
                sender.send(TcpMessage::Message(client_id, msg.into())).await?;
            },
            // an error occurred
            Some(Err(e)) => {
                return Err(e.into());
            }
                // socket was closed
            None => {
                sender.send(TcpMessage::Disconnect(client_id)).await?;
                return Ok(());
            }
        },

        }
    }
}
