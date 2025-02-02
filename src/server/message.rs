use anyhow::bail;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum ServerMessage {
    Ok,
    Err,
    RequestAuthorization,
    MatchBegan,
    Challenged,
    Hint,
    ListOfOpponents,
    Attempt,
    AttemptResult
}

impl TryFrom<u8> for ServerMessage {
    type Error = anyhow::Error;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == ServerMessage::Ok as u8 => Ok(ServerMessage::Ok),
            x if x == ServerMessage::Err as u8 => Ok(ServerMessage::Err),
            x if x == ServerMessage::RequestAuthorization as u8 => Ok(ServerMessage::RequestAuthorization),
            x if x == ServerMessage::MatchBegan as u8 => Ok(ServerMessage::MatchBegan),
            x if x == ServerMessage::Challenged as u8 => Ok(ServerMessage::Challenged),
            x if x == ServerMessage::ListOfOpponents as u8 => Ok(ServerMessage::ListOfOpponents),
            x if x == ServerMessage::Hint as u8 => Ok(ServerMessage::Hint),
            x if x == ServerMessage::Attempt as u8 => Ok(ServerMessage::Attempt),
            x if x == ServerMessage::AttemptResult as u8 => Ok(ServerMessage::AttemptResult),
            _ => bail!("Couldn't convert {} to ServerMessage", v),
        }
    }
}