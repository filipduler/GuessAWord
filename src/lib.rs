mod server;
mod client;
mod utils;
mod bin_writer;
mod bin_reader;


#[cfg(test)]
mod tests {
    use crate::client::{Client, StreamedMessage};
    use crate::server::run_async;

    const PASSWORD: &'static str = "password";

    #[tokio::test]
    async fn it_works_async() {

        _ = tokio::spawn(async move {
            _ = run_async("127.0.0.1:8080", PASSWORD).await;
        });

        //wait for the server to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let mut challenger = Client::connect_async("127.0.0.1:8080".parse().unwrap(), PASSWORD).await.unwrap();

        let mut opponent = Client::connect_async("127.0.0.1:8080".parse().unwrap(), PASSWORD).await.unwrap();

        let opponents = challenger.get_opponents_async().await.unwrap();
        assert_eq!(opponents.len(), 1);

        // challenger first opponent
        let word = "test";
        challenger.request_match_async(opponents[0], word).await.unwrap();

        // opponent should receive a challenged message
        assert_eq!(Some(StreamedMessage::Challenged), opponent.read_streamed_message_async().await.unwrap());

        // sent first attempt
        // opponent should get a negative response because the attempt wasn't correct
        let attempt = "attempt1";
        assert_eq!(false, opponent.send_attempt_async(attempt).await.unwrap());

        // challenger should get the attempt text
        assert_eq!(
            Some(StreamedMessage::Attempt(false, attempt.to_string())),
            challenger.read_streamed_message_async().await.unwrap());

        // challenger sends a hint
        let hint = "maybe try 'test'";
        challenger.send_hint_async(hint).await.unwrap();

        // expect hint on opponents side
        assert_eq!(
            Some(StreamedMessage::Hint(hint.to_string())),
            opponent.read_streamed_message_async().await.unwrap());

        //sent the correct word
        assert_eq!(true, opponent.send_attempt_async(word).await.unwrap());

        // expect done on challenges end
        assert_eq!(
            Some(StreamedMessage::Attempt(true, word.to_string())),
            challenger.read_streamed_message_async().await.unwrap());
    }
}
