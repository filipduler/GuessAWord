use std::{env, io};
use library::{Client, ClientId, StreamedMessage};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    let addr = &args[1];
    let password = &args[2];

    let mut client = library::Client::connect_async(addr.parse()?, password).await?;

    println!("Press '1' to list opponents, '2' to wait for a challenge or 'q' to quit:");
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input == "q" {
        println!("Exiting...");
        return Ok(());
    }

    if input == "1" {

        }
    } else {
        println!("Invalid input.");
    }


    Ok(())
}

async fn process_challenger_flow_async(client: &mut Client) -> anyhow::Result<()> {
    println!("Available Opponents:");
    let opponents = client.get_opponents_async().await?;

    if opponents.is_empty() {
        println!("No Opponents found.");
        return Ok(());
    }

    for id in &opponents {
        println!("Opponent ID: {}", id);
    }
    println!("Enter an opponent ID:");

    let mut opponent_input = String::new();
    io::stdin().read_line(&mut opponent_input)?;
    let opponent_input = opponent_input.trim();

    match opponent_input.parse::<ClientId>() {
        Ok(id) if opponents.contains(&id) => {
            println!("Enter a word for the opponent to guess:");
            let mut word_input = String::new();
            io::stdin().read_line(&mut word_input)?;
            let word_input = word_input.trim();

            client.request_match_async(id, word_input).await?;
            println!("You have challenged an opponent with ID: {}", id);

            loop {
                if let Some(msg) = client.read_streamed_message_async().await? {
                    match msg {
                        StreamedMessage::Attempt(valid, word) => {
                            if valid {
                                println!("Opponent guessed the word!");
                                break;
                            } else {
                                println!("Opponent attempted to guess with the word '{}'", word);
                            }
                        }
                    }
                }
            }
        }
        _ => {
            println!("Invalid opponent ID");
        }
}
