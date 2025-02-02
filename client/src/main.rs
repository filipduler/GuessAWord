use std::{env, io};
use std::sync::Arc;
use anyhow::bail;
use library::{Client, ClientId, StreamedMessage};
use tokio::sync::Mutex;

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
        process_challenger_flow_async(client).await?;
    } else if input == "2" {
        process_opponent_flow_async(client).await?;
    } else {
        println!("Invalid input.");
    }


    Ok(())
}

async fn process_opponent_flow_async(client: Client) -> anyhow::Result<()> {

    let client_mut = Arc::new(Mutex::new(client));

    loop {
        if let Some(msg) = client_mut.lock().await.read_streamed_message_async().await
            .expect("Couldn't read streamed message") {
            match msg {
                StreamedMessage::Challenged => {
                    println!("You have been challenged!");
                    break;
                }
                _ => bail!("Unexpected message"),
            }
        }
    }

    let client_mut_clone = client_mut.clone();
    tokio::spawn(async move {
        loop {
            if let Some(msg) = client_mut_clone.lock().await.read_streamed_message_async().await
                .expect("Couldn't read streamed message") {
                match msg {
                    StreamedMessage::Hint(hint) => {
                        println!("Your challenger sent you a hint: {}", hint);
                    }
                    _ => {}
                }
            }
        }
    });


    println!("Enter an attempt:");
    loop {
        let mut attempt_input = String::new();
        io::stdin().read_line(&mut attempt_input)?;
        let attempt = attempt_input.trim();

        let success = client_mut.lock().await.send_attempt_async(attempt).await?;
        if success {
            println!("You guessed the word!");
            break;
        } else {
            println!("Sadly not the correct word. Try again:");
        }
    }

    Ok(())
}

async fn process_challenger_flow_async(client: Client) -> anyhow::Result<()> {
    let client_mut = Arc::new(Mutex::new(client));

    println!("Available Opponents:");
    let opponents = client_mut.lock().await.get_opponents_async().await?;

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

            client_mut.lock().await.request_match_async(id, word_input).await?;

            println!("You have challenged an opponent with ID: {}", id);

            // read attempts
            let client_mut_clone = client_mut.clone();
            tokio::spawn(async move {
                loop {
                    if let Some(msg) = client_mut_clone.lock().await.read_streamed_message_async().await
                            .expect("Couldn't read streamed message") {
                        match msg {
                            StreamedMessage::Attempt(valid, word) => {
                                if valid {
                                    println!("Opponent guessed the word!");
                                    break;
                                } else {
                                    println!("Opponent attempted to guess with the word '{}'", word);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            });

            loop {
                println!("Enter a hint:");
                let mut hint_input = String::new();
                io::stdin().read_line(&mut hint_input)?;
                let hint_input = word_input.trim();

                client_mut.lock().await.send_hint_async(hint_input).await?;
                println!("Hint '{}' sent.", hint_input);
            }
        }
        _ => {
            println!("Invalid opponent ID");
        }
    }

    Ok(())
}
