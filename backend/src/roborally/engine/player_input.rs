use derive_builder::Builder;

use crate::roborally::state::{ PlayerID, MoveCard };

#[derive(Debug, Clone, Builder)]
pub struct PlayerInput {
    pub player_id: PlayerID,
    pub move_cards: Vec<MoveCard>,
}

impl PlayerInput {
    pub fn new(player_id: PlayerID, mmove: &[MoveCard]) -> PlayerInput {
        PlayerInput {
            player_id,
            move_cards: Vec::from(mmove),
        }
    }
}