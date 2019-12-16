use derive_builder::Builder;

use crate::roborally::state::{ PlayerID, MoveCardID };

#[derive(Debug, Clone, Builder)]
pub struct PlayerInput {
    pub player_id: PlayerID,
    pub register_cards_choices: Vec<MoveCardID>,
}