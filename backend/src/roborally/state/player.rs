#![allow(clippy::trivially_copy_pass_by_ref)]

use derive_builder::Builder;

use super::board::*;
use super::cards::*;

pub type PlayerID = u32;

pub const REGISTER_COUNT: usize = 5;

#[derive(Debug, Clone)]
pub struct Player {
    pub id: PlayerID,
    pub robot: Robot,
    pub registers: Vec<Register>,
    pub program_card_deck: Vec<MoveCard>,
}

impl Player {
    pub fn new(id: PlayerID, robot: Robot) -> Player {
        Player {
            id,
            robot,
            registers: vec![],
            program_card_deck: vec![],
        }
    }

    /**
     * For test purposes
     */
    pub fn new_with_move(id: PlayerID, robot: Robot, move_card: MoveCard) -> Player {
        Player {
            id,
            robot,
            registers: vec![Register{ move_card, locked: false }],
            program_card_deck: vec![],
        }
    }

    pub fn set_program_card_deck(&self, program_card_deck: Vec<MoveCard>) -> Player {
        Player {
            robot: self.robot.clone(),
            program_card_deck,
            registers: self.registers.clone(),
            ..*self
        }
    }
}

pub type RobotID = u32;

#[derive(Debug, Default, Clone, Builder)]
#[builder(default)]
pub struct Robot {
    pub id: RobotID,
    pub damage: u32,
    pub life_tokens: u32,
    pub position: Position,
    pub direction: EDirection,
    pub powered_down: EPoweredDown,
}

impl Robot {
    pub fn set_direction(&self, new_direction: EDirection) -> Robot {
        Robot {
            direction: new_direction,
            ..*self
        }
    }

    pub fn set_position(&self, new_position: Position) -> Robot {
        Robot {
            position: new_position,
            ..*self
        }
    }

    pub fn set_damage(&self, new_damage: u32) -> Robot {
        Robot {
            damage: new_damage,
            ..*self
        }
    }

    pub fn set_powered_down(&self, powered_down: EPoweredDown) -> Robot {
        Robot {
            powered_down,
            ..*self
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EPoweredDown {
    No,
    NextRound,
    Yes
}

impl Default for EPoweredDown {
    fn default() -> EPoweredDown {
        EPoweredDown::No
    }
}

#[derive(Debug, Clone)]
pub struct Register {
    pub move_card: MoveCard,
    pub locked: bool,
}