#![allow(clippy::trivially_copy_pass_by_ref)]

use derive_builder::Builder;

use std::clone::Clone;
use std::sync::Arc;
use std::default::Default;

use super::board::*;
use super::StateError;

#[derive(Debug, Default, Clone)]
pub struct State {
    pub board: Arc<Board>,
    pub players: Vec<Player>,
}

impl State {
    pub fn new(board: Board, players: Vec<Player>) -> State {
        State {
            board: Arc::new(board),
            players: players.into_iter().collect(),
        }
    }

    pub fn get_robot_for(&self, player_id: PlayerID) -> Option<&Robot> {
        self.players.iter()
            .find(|p| p.id == player_id)
            .map(|p| &p.robot)
    }

    pub fn get_robot_or_fail(&self, robot_id: RobotID) -> Result<&Robot, StateError> {
        self.players.iter()
            .find(|p| p.robot.id == robot_id)
            .map(|p| &p.robot)
            .ok_or(StateError::RobotNotFoundID{ robot_id })
    }

    pub fn update_robot_fn<F>(&self, robot_id: RobotID, f: F) -> Result<State, StateError>
        where F: Fn(Robot) -> Robot {
        let old_player_index = self.players.iter()
            .position(|p| p.robot.id == robot_id)
            .ok_or(StateError::RobotNotFoundID{ robot_id })?;

        let mut new_players = self.players.clone();
        let old_player = new_players.remove(old_player_index);

        let mut new_robot = old_player.robot.clone(); 
        let new_robot = f(old_player.robot);
        new_players.push(Player {
            robot: new_robot,
            ..old_player
        });

        Ok(State {
            players: new_players,
            board: self.board.clone(),
        })
    }

    pub fn update_robot(&self, new_robot: Robot) -> Result<State, StateError> {
        let old_player_index = self.players.iter()
            .position(|p| p.robot.id == new_robot.id)
            .ok_or(StateError::RobotNotFoundID{ robot_id: new_robot.id })?;

        let mut new_players = self.players.clone();
        let old_player = new_players.remove(old_player_index);
        new_players.push(Player {
            robot: new_robot,
            ..old_player
        });

        Ok(State {
            players: new_players,
            board: self.board.clone(),
        })
    }

    pub fn find_robot_at(&self, pos: &Position) -> Option<&Robot> {
        self.players.iter()
            .find(|p| p.robot.position == *pos)
            .map(|p| &p.robot)
    }
}

pub type PlayerID = u32;

#[derive(Debug, Clone)]
pub struct Player {
    pub id: PlayerID,
    pub robot: Robot,
}

impl Player {
    pub fn new(id: PlayerID, robot: Robot) -> Player {
        Player {
            id,
            robot,
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