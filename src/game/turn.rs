use crate::config;
use crate::game::PlayerId;
use crate::game::MAX_TURNS;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct Turn {
    player: PlayerId,
    turn: u32,
    phase: Phase,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Phase {
    Start,
    Draw,
    Play,
    End,
}
impl Phase {
    pub fn next(&self) -> Phase {
        match self {
            Phase::Start => Phase::Draw,
            Phase::Draw => Phase::Play,
            Phase::Play => Phase::End,
            Phase::End => Phase::Start,
        }
    }
}
impl Turn {
    pub fn new(player: PlayerId, turn: u32, phase: Phase) -> Turn {
        Turn {
            player,
            turn,
            phase,
        }
    }
    pub fn player(&self) -> PlayerId {
        self.player
    }

    pub fn turn_count(&self) -> u32 {
        self.turn
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// The max amount of time allowed for this turn.
    pub fn get_duration(&self) -> Duration {
        match self.phase {
            Phase::Play => Duration::from_secs(30),
            _ => Duration::from_millis(0),
        }
    }

    /// Creates the next logical player turn or None if no next turn is possable.
    pub fn next(&self) -> Option<Turn> {
        if self.phase == Phase::End {
            // The final phase of the turn.
            let next_player_id = (self.player + 1) % config::active().player_count;
            let turn_count = if next_player_id == 0 {
                self.turn + 1
            } else {
                self.turn
            };
            if turn_count >= config::active().turn_limit {
                None
            } else {
                Some(Turn::new(next_player_id, turn_count, Phase::Start))
            }
        } else {
            Some(Turn::new(self.player, self.turn, self.phase.next()))
        }
    }
}
