#![deny(clippy::single_match)]

use derive_builder::Builder;
use failure::Fail;

use std::slice::Iter;
use std::fmt;
use std::convert;

use crate::game::state::{
    State,
    PlayerID,
    RobotID,
    EDirection,
    EConnection,
    Position,
};

#[derive(Debug, Fail)]
pub enum EngineError {
    #[fail(display = "Robot not found for player with id {}", player_id)]
    RobotNotFound {
        player_id: PlayerID,
    },
    #[fail(display = "Robot with id {} not found", robot_id)]
    RobotNotFoundID {
        robot_id: RobotID,
    },
    #[fail(display = "No position after {:?} {:?}", pos, dir)]
    PositionNotOnBoard {
        pos: Position,
        dir: EDirection,
    }
}

#[derive(Default)]
pub struct Engine {}

impl Engine {
    pub fn new() -> Engine {
        Engine::default()
    }

    pub fn run_register_phase(&self, state: Box<State>, inputs: &MoveInputs) -> Result<Box<State>, EngineError> {
        // Phase:
        // 1. Robots move, in order of Priority
        let mut state = state;
        let move_inputs = inputs.get_sorted_by_priority();
        for move_input in move_inputs {
            let tmove = move_input.mmove.tmove;
            state = self.perform_move(state, move_input.player_id, tmove)?;
        }

        // 2. Board elements move:
        // a. express conveyor belt move 1
        // b. Express conveyor belt and normal conveyor belts move 1 space
        // c. Pusher: push if active (depends on phase)
        // d. Gears rotate

        // 3. Board and robot lasers fire
        // 4. Robots touch flags and place archive markers

        Ok(state)
    }

    fn perform_move(&self, state: Box<State>, player_id: PlayerID, tmove: Box<dyn TMove>) -> Result<Box<State>, EngineError> {
        let mut state = state;
        for smove in tmove.iter() {
            state = self.perform_simple_move(state, player_id, smove)?;
        }
        Ok(state)
    }

    fn perform_simple_move(&self, state: Box<State>, player_id: PlayerID, smove: &ESimpleMove) -> Result<Box<State>, EngineError> {
        if smove.is_turn() {
            let robot = state.get_robot_for(player_id).ok_or(EngineError::RobotNotFound{ player_id })?;
            let new_direction = Engine::map_move_to_direction_change(smove, robot.direction);
            let new_robot = robot.set_direction(new_direction);
            Ok(Box::from(state.update_robot(new_robot)))
        } else {
            let robot = state.get_robot_for(player_id).ok_or(EngineError::RobotNotFound{ player_id })?;
            let direction = Engine::map_move_to_direction_change(smove, robot.direction);
            self.try_to_move_robot(state, player_id, direction)
        }
    }

    fn try_to_move_robot(&self, state: Box<State>, moving_robot_id: RobotID, direction: EDirection) -> Result<Box<State>, EngineError> {
        let mut state = state;
        let board = state.board.clone();

        // Build push stack for pushing other robots away
        let mut push_stack: Vec<RobotID> = vec![];
        push_stack.push(moving_robot_id);

        // 1. Gather move chain (limited by wall)
        loop {
            let robot_id = push_stack.last().unwrap(); // TODO logic error!
            let robot = state.get_robot(*robot_id).unwrap();
            let from = &robot.position;

            // Handle different neighbor connection
            let to = match board.get_neighbor_in(from, direction) {
                None => {
                    return Err(EngineError::PositionNotOnBoard { pos: *from, dir: direction })
                },
                Some(EConnection::Walled) => {
                    // No further chaining or movement possible: we're done here
                    return Ok(state)
                },
                Some(EConnection::Free(to)) => to,
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
            let robot = state.get_robot(*robot_id)
                .ok_or(EngineError::RobotNotFoundID{ robot_id: *robot_id })?;
            
            let to = match board.get_neighbor_in(&robot.position, direction) {
                None => {
                    return Err(EngineError::PositionNotOnBoard { pos: robot.position, dir: direction })
                },
                Some(EConnection::Walled) => {
                    // Cannot move
                    continue;
                },
                Some(EConnection::Free(to)) => to,
            };

            // Actual move TODO Should field do this, too?
            let new_robot = robot.set_position(to);
            state = Box::from(state.update_robot(new_robot));
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
    fn box_clone(&self) -> Box<dyn TMove>;
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

#[derive(Debug, Clone)]
pub struct SimpleMove {
    chain: Vec<ESimpleMove>,
}

impl SimpleMove {
    pub fn new(simple_moves: &[ESimpleMove]) -> Box<SimpleMove> {
        Box::from(SimpleMove {
            chain: simple_moves.to_vec(),
        })
    }

    pub fn single(mmove: ESimpleMove) -> Box<SimpleMove> {
        SimpleMove::new(&[mmove])
    }
}

impl TMove for SimpleMove {
    fn iter(&self) -> Iter<ESimpleMove> {
        self.chain.iter()
    }
    fn box_clone(&self) -> Box<dyn TMove> {
        Box::new((*self).clone())
    }
}

#[derive(Debug, Clone)]
pub struct MoveCard {
    pub priority: i32,
    pub tmove: Box<dyn TMove>,
}

impl MoveCard {
    pub fn new(priority: i32, tmove: Box<dyn TMove>) -> MoveCard {
        MoveCard {
            priority,
            tmove,
        }
    }
}

#[derive(Debug, Clone, Builder)]
pub struct MoveInput {
    pub player_id: PlayerID,
    pub mmove: MoveCard,
}

impl MoveInput {
    pub fn new(player_id: PlayerID, mmove: MoveCard) -> MoveInput {
        MoveInput {
            player_id,
            mmove,
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
    pub fn get_sorted_by_priority(&self) -> Vec<MoveInput> {
        let mut moves = self.move_inputs.as_slice().to_vec();
        moves.sort_by(|a, b| a.mmove.priority.partial_cmp(&b.mmove.priority).unwrap());
        moves
    }
}

// pub type RoundInputs = Vec<RoundInput>;
// pub struct RoundInput {
//     pub player_id: PlayerID,
//     pub power_down: bool,
//     pub cards: Vec<MoveCard>,
// }

// #[derive(Debug, Builder)]
// pub struct PosDirMove {
//     pub position: Position,
//     pub direction: EDirection,
//     pub moves: Vec<ESimpleMove>,
// }

// pub fn create_test_state(pdm1: PosDirMove, pdm2: PosDirMove) -> State {
//     let robot1 = RobotBuilder::default()
//         .id(0)
//         .position(Position::new(2, 2))
//         .direction(EDirection::NORTH)
//         .build().unwrap();
//     let player1 = Player::new(0, robot1);

//     let robot2 = RobotBuilder::default()
//         .id(1)
//         .position(Position::new(4, 4))
//         .direction(EDirection::EAST)
//         .build().unwrap();
//     let player2 = Player::new(1, robot2);
// }

#[cfg(test)]
mod test {
    use crate::game::state::*;
    use crate::game::engine::*;

    fn create_state() -> (Board, Vec<Player>, MoveInputs) {
        // State
        let robot1 = RobotBuilder::default()
            .id(0)
            .position(Position::new(2, 2))
            .direction(EDirection::NORTH)
            .build().unwrap();
        let player1 = Player::new(0, robot1);

        let robot2 = RobotBuilder::default()
            .id(1)
            .position(Position::new(4, 4))
            .direction(EDirection::EAST)
            .build().unwrap();
        let player2 = Player::new(1, robot2);

        // Inputs
        let move_forward = SimpleMove::single(ESimpleMove::Forward);
        let move_card1 = MoveCard::new(1, move_forward);
        let move_input1 = MoveInput::new(player1.id, move_card1);

        let move_left_forward = SimpleMove::new(&[ESimpleMove::TurnLeft, ESimpleMove::Forward]);
        let move_card2 = MoveCard::new(2, move_left_forward);
        let move_input2 = MoveInput::new(player2.id, move_card2);
        
        let ins = vec![move_input1, move_input2];
        let inputs = MoveInputs::from(ins.as_slice());

        let board = Board::new_empty_board(5, 5);
        (board, vec![player1, player2], inputs)
    }

    #[test]
    fn test_simple_move() -> Result<(), Box<EngineError>> {
        let (board, players, inputs) = create_state();
        let state = Box::from(State::new(board, players));
        
        let engine = Engine::default();
        let actual_state = engine.run_register_phase(state, &inputs)?;

        let actual_robot1 = actual_state.get_robot_for(0).unwrap();
        let actual_robot2 = actual_state.get_robot_for(1).unwrap();
        assert_eq!(actual_robot1.direction, EDirection::NORTH, "robot1 direction");
        assert_eq!(actual_robot1.position, Position { x: 2, y: 1 }, "robot1 position");
        assert_eq!(actual_robot2.direction, EDirection::NORTH, "robot2 direction");
        assert_eq!(actual_robot2.position, Position { x: 4, y: 3 }, "robot2 position");

        Ok(())
    }

    #[test]
    fn test_wall_blocks() -> Result<(), Box<EngineError>> {
        // Board
        let board = Board {
            tiles: vec![
                Tile {
                    position: Position { x: 0, y: 0 },
                    ttype: ETileType::Free,
                    walls: vec![EDirection::SOUTH, EDirection::EAST],
                },
                Tile {
                    position: Position { x: 1, y: 0 },
                    ttype: ETileType::Free,
                    walls: vec![],
                },
                Tile {
                    position: Position { x: 0, y: 1 },
                    ttype: ETileType::Free,
                    walls: vec![],
                },
                Tile {
                    position: Position { x: 1, y: 1 },
                    ttype: ETileType::Free,
                    walls: vec![],
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
        let player1 = Player::new(player_id1, robot1);

        let player_id2: u32 = 1;
        let robot2_pos = Position::new(1, 0);
        let robot2 = RobotBuilder::default()
            .id(1)
            .position(robot2_pos)
            .direction(EDirection::WEST)
            .build().unwrap();
        let player2 = Player::new(player_id2, robot2);
        let players = vec![player1, player2];

        // Inputs
        let move_forward = SimpleMove::single(ESimpleMove::Forward);
        let move_card1 = MoveCard::new(1, move_forward.clone());
        let move_input1 = MoveInput::new(player_id1, move_card1);

        let move_card2 = MoveCard::new(2, move_forward);
        let move_input2 = MoveInput::new(player_id2, move_card2);

        let ins = vec![move_input1, move_input2];
        let inputs = MoveInputs::from(ins.as_slice());

        // State
        let state = Box::from(State::new(board, players));
        
        let engine = Engine::default();
        let actual_state = engine.run_register_phase(state, &inputs)?;

        let actual_robot1 = actual_state.get_robot_for(0).unwrap();
        let actual_robot2 = actual_state.get_robot_for(1).unwrap();
        assert_eq!(actual_robot1.direction, EDirection::NORTH, "robot1 direction");
        assert_eq!(actual_robot1.position, robot1_pos, "robot1 position");
        assert_eq!(actual_robot2.direction, EDirection::WEST, "robot2 direction");
        assert_eq!(actual_robot2.position, robot2_pos, "robot2 position");

        Ok(())
    }
}