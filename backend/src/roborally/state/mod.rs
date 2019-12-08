#![allow(clippy::trivially_copy_pass_by_ref)]

use failure::Fail;

use std::sync::Arc;

mod cards;
mod board;
mod player;

pub use board::*;
pub use cards::*;
pub use player::*;

#[derive(Debug, Fail)]
pub enum StateError {
    #[fail(display = "Robot with id {} not found", robot_id)]
    RobotNotFoundID {
        robot_id: RobotID,
    },
    #[fail(display = "Player not found for id {}", player_id)]
    PlayerNotFound {
        player_id: PlayerID,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RoundPhase {
    /**
     * Cards are dealt + some maintenance stuff (power down)
     */
    PREPARATION,

    /**
     * The program phase: players fill their registers with their dealt cards and may announce power down
     */
    PROGRAM,

    /**
     * The robots are moved according to the users programs and all resulting effects are executed
     */
    EXECUTE,

    /**
     * Robots are repaired and option cards are drawn and executed
     */
    CLEANUP
}

impl Default for RoundPhase {
    fn default() -> Self {
        RoundPhase::PREPARATION
    }
}

#[derive(Debug, Clone, Default)]
pub struct State {
    pub board: Arc<Board>,
    pub players: Vec<Player>,
    pub deck: ProgramCardDeck,
    pub phase: RoundPhase,
}

impl State {
    pub fn new(board: Board, players: Vec<Player>) -> State {
        let config = ProgramCardDeckConfig::default();
        let gen = ProgramCardDeckGenerator::new();
        State {
            board: Arc::new(board),
            players: players.into_iter().collect(),
            deck: gen.generate_program_deck(config),
            phase: RoundPhase::PREPARATION,
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

    pub fn update_robot_fn<T>(&self, robot_id: RobotID, transform: T) -> Result<State, StateError>
        where T: Fn(Robot) -> Robot {
        let old_player_index = self.players.iter()
            .position(|p| p.robot.id == robot_id)
            .ok_or(StateError::RobotNotFoundID{ robot_id })?;

        let mut new_players = self.players.clone();
        let old_player = new_players[old_player_index].clone();
        let new_robot = transform(old_player.robot);
        new_players[old_player_index] = Player {
            robot: new_robot,
            ..old_player
        };

        Ok(State {
            players: new_players,
            board: self.board.clone(),
            deck: self.deck.clone(),
            ..*self
        })
    }

    pub fn update_robot(&self, new_robot: Robot) -> Result<State, StateError> {
        let old_player_index = self.players.iter()
            .position(|p| p.robot.id == new_robot.id)
            .ok_or(StateError::RobotNotFoundID{ robot_id: new_robot.id })?;

        let mut new_players = self.players.clone();
        new_players[old_player_index] = Player {
            robot: new_robot,
            ..new_players[old_player_index].clone()
        };

        Ok(State {
            players: new_players,
            board: self.board.clone(),
            deck: self.deck.clone(),
            ..*self
        })
    }

    pub fn update_player(&self, new_player: Player) -> Result<State, StateError> {
        let old_player_index = self.players.iter()
            .position(|p| p.id == new_player.id)
            .ok_or(StateError::PlayerNotFound{ player_id: new_player.id })?;

        let mut new_players = self.players.clone();
        new_players[old_player_index] = new_player;

        Ok(State {
            players: new_players,
            board: self.board.clone(),
            deck: self.deck.clone(),
            ..*self
        })
    }

    pub fn set_deck(&self, new_deck: ProgramCardDeck) -> State {
        State {
            deck: new_deck,
            players: self.players.clone(),
            board: self.board.clone(),
            ..*self
        }
    }

    pub fn set_phase(&self, new_phase: RoundPhase) -> Box<State> {
        Box::from(State {
            phase: new_phase,
            players: self.players.clone(),
            board: self.board.clone(),
            deck: self.deck.clone(),
        })
    }

    pub fn find_robot_at(&self, pos: &Position) -> Option<&Robot> {
        self.players.iter()
            .find(|p| p.robot.position == *pos)
            .map(|p| &p.robot)
    }
    
    pub fn get_player_cards_sorted_by_priority(&self) -> Vec<(PlayerID, MoveCard)> {
        let mut moves: Vec<(PlayerID, MoveCard)> = self.players.iter()
            .map(|p| {
                let mut mcs: Vec<(PlayerID, MoveCard)> = vec![];
                for r in &p.registers {
                    mcs.push((p.id, r.move_card.clone()));
                }
                mcs
            })
            .flatten()
            .collect();
        moves.sort_by(|a, b| a.1.priority.partial_cmp(&b.1.priority).unwrap());
        moves
    }
}