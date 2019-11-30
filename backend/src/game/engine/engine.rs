use failure::Fail;

use crate::game::state::{ PlayerID };

#[derive(Debug, Fail)]
pub enum EngineError {
    #[fail(display = "Invalid player id: {}", player_id)]
    Invalid {
        player_id: PlayerID,
    }
}

pub struct Engine {

}

impl Engine {
    
}