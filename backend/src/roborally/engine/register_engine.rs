#![deny(clippy::single_match)]
use std::slice::Iter;
use std::fmt;

use failure::Fail;

use crate::roborally::state::{
    State,
    StateError,
    PlayerID,
    RobotID,
    EDirection,
    EConnection
};

#[derive(Debug, Fail)]
pub enum RegisterEngineError {
    #[fail(display = "Move error: {}", msg)]
    GenericAlgorithmError {
        msg: String,
    },
    #[fail(display = "State error: {}", err)]
    StateError {
        err: StateError,
    }
}

impl From<StateError> for RegisterEngineError {
    fn from(err: StateError) -> Self {
        RegisterEngineError::StateError{ err }
    }
}

#[derive(Default)]
pub struct RegisterEngine {}

impl RegisterEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn execute_registers(&self, state: Box<State>) -> Result<Box<State>, RegisterEngineError> {
        let mut state = state;

        for register_index in 0..state.register_count() {
            state = self.run_register_phase(state, register_index)?;
        }
        Ok(state)
    }

    fn run_register_phase(&self, state: Box<State>, register_index: usize) -> Result<Box<State>, RegisterEngineError> {
        let mut state = state;

        // 1. Robots move, in order of Priority
        let player_move_cards = state.get_register_cards_sorted_by_priority(register_index)?;
        for player_card in player_move_cards {
            let tmove = player_card.1.tmove;
            state = self.perform_move(state, player_card.0, tmove)?;
        }

        // 2. Board elements move:
        // a. express conveyor belt move 1
        // b. Express conveyor belt and normal conveyor belts move 1 space
        // c. Pusher: push if active (depends on phase)
        // d. Gears rotate

        // 3. Board and robot lasers fire
        // 4. Robots on flags or repair site: update archive markers

        Ok(state)
    }

    fn perform_move(&self, state: Box<State>, player_id: PlayerID, tmove: Box<dyn TMove>) -> Result<Box<State>, RegisterEngineError> {
        let mut state = state;
        for smove in tmove.iter() {
            let player = state.get_player_or_fail(player_id)?;
            if !player.is_active() {
                // The robot died because he moved off platform: Don't move it anymore!
                return Ok(state)
            }

            state = self.perform_simple_move(state, player_id, smove)?;
        }
        Ok(state)
    }

    fn perform_simple_move(&self, state: Box<State>, player_id: PlayerID, smove: &ESimpleMove) -> Result<Box<State>, RegisterEngineError> {
        if smove.is_turn() {
            let robot = state.get_robot_by_player_id_or_fail(player_id)?;
            let new_direction = Self::map_move_to_direction_change(smove, robot.direction);
            let new_robot = robot.set_direction(new_direction);
            Ok(state.update_robot(new_robot)?)
        } else {
            let robot = state.get_robot_by_player_id_or_fail(player_id)?;
            let robot_id = robot.id;
            let direction = Self::map_move_to_direction_change(smove, robot.direction);
            self.try_to_move_robot(state, robot_id, direction)
        }
    }

    fn try_to_move_robot(&self, state: Box<State>, moving_robot_id: RobotID, direction: EDirection) -> Result<Box<State>, RegisterEngineError> {
        let mut state = state;
        let board = state.board.clone();

        // Build push stack for pushing other robots away
        let mut push_stack: Vec<RobotID> = vec![];
        push_stack.push(moving_robot_id);

        // 1. Gather move chain (limited by wall)
        loop {
            let robot_id = push_stack.last()
                .ok_or(RegisterEngineError::GenericAlgorithmError {
                    msg: String::from("Expected stack to not be empty!"),
                })?;
            let robot = state.get_robot_by_id_or_fail(*robot_id)?;
            let from = &robot.position;

            // Handle different neighbor connection
            let to = match board.get_neighbor_in(from, direction)? {
                EConnection::Free(to) => to,
                EConnection::Walled => {
                    // No further chaining or movement possible: we're done here
                    return Ok(state)
                },
                EConnection::OffPlatform(_) => {
                    // Ohoh, someone's going to die...
                    // This discontinues the chain as well. Multiple robots may end up on that position!
                    break;
                },
            };

            // Watch out for the next robot for our chain
            let robot_in_my_way = state.find_robot_at(&to);
            if robot_in_my_way.is_none() {
                // Chain discontinued: End here
                break;
            }

            // Another one: Try to find the next!
            push_stack.push(robot_in_my_way.unwrap().id);
            continue;
        }

        // 2. Try to actually move
        while !push_stack.is_empty() {
            let robot_id = push_stack.last().unwrap();
            let robot = state.get_robot_by_id_or_fail(*robot_id)?;
            
            let new_robot = match board.get_neighbor_in(&robot.position, direction)? {
                EConnection::Free(to) => {
                    robot.set_position(to)
                },
                EConnection::Walled => {
                    // Cannot move
                    continue;
                },
                EConnection::OffPlatform(to) => {
                    // NOOOOoooooooohhh........
                    robot.set_position(to)
                        .die()
                },
            };

            // Actual move
            state = state.update_robot(new_robot)?;
            push_stack.pop();
        }
        Ok(state)
    }

    fn map_move_to_direction_change(smove: &ESimpleMove, dir: EDirection) -> EDirection {
        match smove {
            // Turn
            ESimpleMove::TurnLeft => dir.turn_left(),
            ESimpleMove::TurnRight => dir.turn_right(),
            ESimpleMove::UTurn => dir.turn_around(),

            // Moves
            ESimpleMove::Backward => dir.turn_around(),
            ESimpleMove::StepLeft => dir.turn_left(),
            ESimpleMove::StepRight => dir.turn_right(),
            ESimpleMove::Forward => dir,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ESimpleMove {
    Forward,
    Backward,
    StepLeft,
    StepRight,
    
    TurnRight,
    TurnLeft,
    UTurn,
}

impl ESimpleMove {
    pub fn is_turn(&self) -> bool {
        match self {
            ESimpleMove::TurnRight => true,
            ESimpleMove::TurnLeft => true,
            ESimpleMove::UTurn => true,
            _ => false,
        }
    }
}

pub trait TMove {
    fn iter(&self) -> Iter<ESimpleMove>;

    /**
     * Helper method for being able to clone() Trait Objects
     * Reference: https://users.rust-lang.org/t/solved-is-it-possible-to-clone-a-boxed-trait-object/1714/6
     */
    fn box_clone(&self) -> Box<dyn TMove + Send>;
}

impl Clone for Box<dyn TMove> {
    fn clone(&self) -> Box<dyn TMove> {
        self.box_clone()
    }
}
impl fmt::Debug for dyn TMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let move_strs: Vec<String> = self.iter()
            .map(|m| format!("{:?}", m))
            .collect();
        write!(f, "TMove {{ {} }}", move_strs.join(", "))
    }
}

#[cfg(test)]
mod test {
    use failure::Error;
    use crate::roborally::state::*;
    use crate::roborally::engine::register_engine::*;

    fn create_state() -> Result<(Board, Vec<Player>), Error> {
        // State
        let robot1 = RobotBuilder::default()
            .id(0)
            .position(Position::new(2, 2))
            .direction(EDirection::NORTH)
            .build().unwrap();
        let player1 = Player::new_with_move(0, robot1, MoveCard::new_from_moves(0, 1, &[ESimpleMove::Forward]));

        let robot2 = RobotBuilder::default()
            .id(1)
            .position(Position::new(4, 4))
            .direction(EDirection::EAST)
            .build().unwrap();
        let player2 = Player::new_with_move(1, robot2, MoveCard::new_from_moves(1, 2, &[ESimpleMove::TurnLeft, ESimpleMove::Forward]));

        let board = Board::load_board_by_name("empty-5x5")?;
        Ok((board, vec![player1, player2]))
    }

    #[test]
    fn test_simple_move() -> Result<(), Error> {
        let (board, players) = create_state()?;
        let state = State::new_with_random_deck(board, players);
        
        let engine = RegisterEngine::default();
        let actual_state = engine.execute_registers(state)?;

        let actual_robot1 = actual_state.get_robot_by_player_id_or_fail(0)?;
        let actual_robot2 = actual_state.get_robot_by_player_id_or_fail(1)?;
        assert_eq!(actual_robot1.direction, EDirection::NORTH, "robot1 direction");
        assert_eq!(actual_robot1.position, Position { x: 2, y: 1 }, "robot1 position");
        assert_eq!(actual_robot2.direction, EDirection::NORTH, "robot2 direction");
        assert_eq!(actual_robot2.position, Position { x: 4, y: 3 }, "robot2 position");

        Ok(())
    }

    #[test]
    fn test_dead_robot_does_not_move() -> Result<(), Error> {
        let (board, _) = create_state()?;

        // Players + Robots
        let player_id1: u32 = 0;
        let robot1_pos = Position::new(0, 0);
        let robot1 = RobotBuilder::default()
            .id(0)
            .position(robot1_pos.clone())
            .direction(EDirection::NORTH)
            .build().unwrap();
        let player1 = Player::new_with_move(player_id1, robot1, MoveCard::new_from_moves(0, 1, &[ESimpleMove::Forward, ESimpleMove::Forward, ESimpleMove::Forward]));

        let player_id2: u32 = 1;
        let robot2_pos = Position::new(1, 0);
        let robot2 = RobotBuilder::default()
            .id(1)
            .position(robot2_pos)
            .direction(EDirection::SOUTH)
            .build().unwrap();
        let player2 = Player::new_with_move(player_id2, robot2, MoveCard::new_from_moves(1, 2, &[ESimpleMove::Forward]));
        let players = vec![player1, player2];

        // State
        let state = State::new_with_random_deck(board, players);
        
        let engine = RegisterEngine::default();
        let actual_state = engine.execute_registers(state)?;

        let actual_robot1 = actual_state.get_robot_by_player_id_or_fail(0)?;
        let actual_robot2 = actual_state.get_robot_by_player_id_or_fail(1)?;
        assert_eq!(actual_robot1.direction, EDirection::NORTH, "robot1 direction");
        assert_eq!(actual_robot1.position, Position { x: 0, y: -1 }, "robot1 position");
        assert_eq!(actual_robot2.direction, EDirection::SOUTH, "robot2 direction");
        assert_eq!(actual_robot2.position, Position { x: 1, y: 1 }, "robot2 position");

        Ok(())
    }

    #[test]
    fn test_wall_blocks() -> Result<(), Error> {
        // Board
        let board = Board {
            tiles: vec![
                Tile {
                    position: Position { x: 0, y: 0 },
                    ttype: ETileType::Regular,
                    walls: vec![EDirection::SOUTH, EDirection::EAST],
                    start_position_id: None,
                },
                Tile {
                    position: Position { x: 1, y: 0 },
                    ttype: ETileType::Regular,
                    walls: vec![],
                    start_position_id: None,
                },
                Tile {
                    position: Position { x: 0, y: 1 },
                    ttype: ETileType::Regular,
                    walls: vec![],
                    start_position_id: None,
                },
                Tile {
                    position: Position { x: 1, y: 1 },
                    ttype: ETileType::Regular,
                    walls: vec![],
                    start_position_id: None,
                },
            ],
            size_x: 2,
            size_y: 2,
        };

        // Players + Robots
        let player_id1: u32 = 0;
        let robot1_pos = Position::new(0, 1);
        let robot1 = RobotBuilder::default()
            .id(0)
            .position(robot1_pos.clone())
            .direction(EDirection::NORTH)
            .build().unwrap();
        let player1 = Player::new_with_move(player_id1, robot1, MoveCard::new_from_moves(0, 1, &[ESimpleMove::Forward]));

        let player_id2: u32 = 1;
        let robot2_pos = Position::new(1, 0);
        let robot2 = RobotBuilder::default()
            .id(1)
            .position(robot2_pos)
            .direction(EDirection::WEST)
            .build().unwrap();
        let player2 = Player::new_with_move(player_id2, robot2, MoveCard::new_from_moves(1, 2, &[ESimpleMove::Forward]));
        let players = vec![player1, player2];

        // State
        let state = State::new_with_random_deck(board, players);
        
        let engine = RegisterEngine::default();
        let actual_state = engine.execute_registers(state)?;

        let actual_robot1 = actual_state.get_robot_by_player_id_or_fail(0)?;
        let actual_robot2 = actual_state.get_robot_by_player_id_or_fail(1)?;
        assert_eq!(actual_robot1.direction, EDirection::NORTH, "robot1 direction");
        assert_eq!(actual_robot1.position, robot1_pos, "robot1 position");
        assert_eq!(actual_robot2.direction, EDirection::WEST, "robot2 direction");
        assert_eq!(actual_robot2.position, robot2_pos, "robot2 position");

        Ok(())
    }
}