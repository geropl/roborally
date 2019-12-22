use derive_builder::Builder;

use crate::roborally::state::{ PlayerID, MoveCardID, StartPositionID };

#[derive(Debug, Clone, Builder)]
pub struct ProgramInput {
    pub player_id: PlayerID,
    pub register_cards_choices: Vec<MoveCardID>,
}

#[derive(Debug, Clone, Builder)]
pub struct StartPositionInput {
    pub player_id: PlayerID,
    pub start_position_id: StartPositionID,
}