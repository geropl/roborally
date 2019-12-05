use derive_builder::Builder;

use std::convert;

use crate::roborally::state::{ PlayerID, MoveCard };

#[derive(Debug, Clone, Builder)]
pub struct MoveInput {
    pub player_id: PlayerID,
    pub move_cards: Vec<MoveCard>,
}

impl MoveInput {
    pub fn new(player_id: PlayerID, mmove: &[MoveCard]) -> MoveInput {
        MoveInput {
            player_id,
            move_cards: Vec::from(mmove),
        }
    }
}

#[derive(Debug, Clone, Builder)]
pub struct MoveInputs {
    move_inputs: Vec<MoveInput>,
}

impl convert::From<&[MoveInput]> for MoveInputs {
    fn from(inputs: &[MoveInput]) -> Self {
        MoveInputs {
            move_inputs: Vec::from(inputs),
        }
    }
}

impl MoveInputs {
    pub fn get_player_cards_sorted_by_priority(&self) -> Vec<(PlayerID, MoveCard)> {
        let mut moves: Vec<(PlayerID, MoveCard)> = self.move_inputs.iter()
            .map(|mi| {
                let mut mcs: Vec<(PlayerID, MoveCard)> = vec![];
                for mc in &mi.move_cards {
                    mcs.push((mi.player_id, mc.clone()));
                }
                mcs
            })
            .flatten()
            .collect();
        moves.sort_by(|a, b| a.1.priority.partial_cmp(&b.1.priority).unwrap());
        moves
    }
}