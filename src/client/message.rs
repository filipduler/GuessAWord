use anyhow::bail;

#[derive(Debug, Clone, PartialEq)]
#[repr(u8)]
pub enum ClientMessage {
    Authorization,
    ListOfOpponents,
    RequestMatch,
    SendHint,
    SendAttempt
}

impl TryFrom<u8> for ClientMessage {
    type Error = anyhow::Error;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == ClientMessage::Authorization as u8 => Ok(ClientMessage::Authorization),
            x if x == ClientMessage::ListOfOpponents as u8 => Ok(ClientMessage::ListOfOpponents),
            x if x == ClientMessage::RequestMatch as u8 => Ok(ClientMessage::RequestMatch),
            x if x == ClientMessage::SendHint as u8 => Ok(ClientMessage::SendHint),
            x if x == ClientMessage::SendAttempt as u8 => Ok(ClientMessage::SendAttempt),
            _ => bail!("Couldn't convert {} to ClientMessage", v),
        }
    }
}