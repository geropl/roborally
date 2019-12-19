#![allow(clippy::module_inception)]

use failure::Fail;

mod cards;
mod board;
mod player;
mod state;
mod game_state;
mod board_parser;

pub use board::*;
pub use cards::*;
pub use player::*;
pub use state::*;
pub use game_state::*;
pub use board_parser::*;

#[derive(Debug, Fail)]
pub enum StateError {
    #[fail(display = "Robot with id {} not found", robot_id)]
    RobotNotFoundID {
        robot_id: RobotID,
    },
    #[fail(display = "Robot not found for player with id: {}", player_id)]
    RobotNotFoundPlayerID {
        player_id: PlayerID,
    },
    #[fail(display = "Player not found for id: {}", player_id)]
    PlayerNotFound {
        player_id: PlayerID,
    },
    #[fail(display = "Register program card not set: {}", player_id)]
    EmptyProgramRegister {
        player_id: PlayerID,
    },
    #[fail(display = "No round found for id: {:?}", round_id)]
    GameStateMissingRound {
        round_id: Option<RoundID>,
    },
    #[fail(display = "Round not found for id: {}", round_id)]
    RoundNotFound {
        round_id: RoundID,
    },
    #[fail(display = "Double input for player {}!", player_id)]
    DoublePlayerInput {
        player_id: PlayerID,
    },
    #[fail(display = "Invalid program card choice ({}) by player {}!", move_card_id, player_id)]
    InvalidProgramCardChoice {
        player_id: PlayerID,
        move_card_id: MoveCardID,
    },
    #[fail(display = "Position off board: {:?}", position)]
    PositionOffBoard {
        position: Position,
    },
}
