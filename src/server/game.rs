use std::collections::HashMap;
use anyhow::bail;
use crate::utils::ClientId;

pub struct Game {
    players: HashMap<ClientId, Player>,
    matches: Vec<Match>,
    // WARN: should be a hash of the password
    password: String,
}

pub struct Player {
    id: ClientId,
    state: PlayerState,
}

pub struct Match {
    challenger_id: ClientId,
    opponent_id: ClientId,
    word: String,
}

impl Match {
    pub fn new(challenger_id: ClientId, opponent_id: ClientId, word: String) -> Self {
        Self {
            challenger_id,
            opponent_id,
            word
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlayerState {
    Available,
    InMatch
}

impl Game {
    pub fn new(password: &str) -> Game {
        Game {
            players: HashMap::new(),
            matches: Vec::new(),
            password: password.to_string(),
        }
    }

    pub fn authorize(&self, password: &str) -> bool {
        self.password == password
    }

    pub fn add_player(&mut self, id: ClientId) {
        self.players.insert(id, Player {
            id,
            state: PlayerState::Available,
        });
    }

    pub fn opponents_id(&mut self, challenger_id: ClientId) -> anyhow::Result<ClientId> {
        if let Some(m) = self.matches.iter().find(|x| x.challenger_id == challenger_id) {
            return Ok(m.opponent_id)
        }
        bail!("Match not found with challenger id: {}", challenger_id)
    }

    pub fn challenger_id(&mut self, opponent_id: ClientId) -> anyhow::Result<ClientId> {
        if let Some(m) = self.matches.iter().find(|x| x.opponent_id == opponent_id) {
            return Ok(m.challenger_id)
        }
        bail!("Match not found with opponent id: {}", opponent_id)
    }

    pub fn validate_word(&mut self, opponent_id: ClientId, word: &str) -> anyhow::Result<bool> {
        if let Some(m) = self.matches.iter().find(|x| x.opponent_id == opponent_id) {
            return Ok(word.eq_ignore_ascii_case(&m.word));
        }
        bail!("Match not found with opponent id: {}", opponent_id)
    }

    pub fn remove_player(&mut self, id: ClientId) {
        self.players.remove(&id);

        // remove the matches where the client is in
        self.matches.retain(|m| m.opponent_id != id && m.challenger_id != id);
    }

    pub fn begin_match(
        &mut self,
        challenger_id: ClientId,
        opponent_id: ClientId,
        word: &str) -> anyhow::Result<()> {

        self.mark_player_in_match(challenger_id)?;
        self.mark_player_in_match(opponent_id)?;

        self.matches.push(Match::new(challenger_id, opponent_id, word.to_string()));

        Ok(())
    }

    fn mark_player_in_match(&mut self, id: ClientId) -> anyhow::Result<()> {
        if let Some(player) = self.players.get_mut(&id) {
            if player.state != PlayerState::Available {
                bail!("Player {} not available.", id);
            }
            player.state = PlayerState::InMatch;
        } else {
            bail!("Player {} not found.", id);
        }

        Ok(())
    }


    pub fn list_of_opponents(&self) -> Vec<ClientId> {
        self.players
            .values()
            .filter(|p| p.state == PlayerState::Available)
            .map(|p| p.id)
            .collect()
    }
}
