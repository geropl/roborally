use failure::Fail;

use crate::roborally::state;
use crate::roborally::engine::player_input;
use crate::roborally::engine::register_engine;

tonic::include_proto!("protocol");

// Protocol -> state
#[derive(Debug, Fail)]
pub enum ProtocolError {
    // #[fail(display = "Invalid player input for player: {}", player_id)]
    // InvalidPlayerInput {
    //     player_id: state::PlayerID,
    // },
    #[fail(display = "Missing player input!")]
    MissingPlayerInput {
    },
    #[fail(display = "Expected one of enum {}, found value {}!", enum_name, value)]
    WrongEnumValue {
        enum_name: String,
        value: i32,
    },
}

impl player_input::PlayerInput {
    pub fn parse_from(player_input: Option<PlayerInput>) -> Result<player_input::PlayerInput, ProtocolError> {
        let player_input = player_input.ok_or(ProtocolError::MissingPlayerInput{})?;

        let move_cards: Result<Vec<state::MoveCard>, _> = player_input.move_cards.iter()
            .map(state::MoveCard::parse_from)
            .collect();
        Ok(player_input::PlayerInput {
            player_id: player_input.player_id,
            move_cards: move_cards?, 
        })
    }
}

impl state::MoveCard {
    fn parse_from(move_card: &MoveCard) -> Result<state::MoveCard, ProtocolError> {
        let simple_moves: Result<Vec<register_engine::ESimpleMove>, _> = move_card.moves.iter()
            .map(|mmove_i32| register_engine::ESimpleMove::parse_from(*mmove_i32))
            .collect();
        let simple_moves = simple_moves?;
        Ok(state::MoveCard::new_from_moves(move_card.priority, &simple_moves))
    }
}

impl register_engine::ESimpleMove {
    fn parse_from(mmove_i32: i32) -> Result<register_engine::ESimpleMove, ProtocolError> {
        match ESimpleMove::from_i32(mmove_i32) {
            None => Err(ProtocolError::WrongEnumValue{
                enum_name: String::from("ESimpleMove"),
                value: mmove_i32,
            }),
            Some(m) => Ok(register_engine::ESimpleMove::from(m)),
        }
    }
}

impl From<ESimpleMove> for register_engine::ESimpleMove {
    fn from(mmove: ESimpleMove) -> register_engine::ESimpleMove {
        match mmove {
            ESimpleMove::Forward => register_engine::ESimpleMove::Forward,
            ESimpleMove::Backward => register_engine::ESimpleMove::Backward,
            ESimpleMove::StepLeft => register_engine::ESimpleMove::StepLeft,
            ESimpleMove::StepRight => register_engine::ESimpleMove::StepRight,
            
            ESimpleMove::TurnRight => register_engine::ESimpleMove::TurnRight,
            ESimpleMove::TurnLeft => register_engine::ESimpleMove::TurnLeft,
            ESimpleMove::UTurn => register_engine::ESimpleMove::UTurn,
        }
    }
}

// State -> protocol
impl From<&Box<state::State>> for GameState {
    fn from(state: &Box<state::State>) -> GameState {
        use std::borrow::Borrow;

        let players: Vec<Player> = state.players.iter()
            .map(Player::from)
            .collect();
        GameState {
            board: Some(Board::from(state.board.borrow())),
            players,
        }
    }
}

impl From<&state::State> for GameState {
    fn from(state: &state::State) -> GameState {
        use std::borrow::Borrow;

        let players: Vec<Player> = state.players.iter()
            .map(Player::from)
            .collect();
        GameState {
            board: Some(Board::from(state.board.borrow())),
            players,
        }
    }
}

impl From<&state::Player> for Player {
    fn from(player: &state::Player) -> Player {
        Player {
            id: player.id,
            robot: Some(Robot::from(&player.robot)),
        }
    }
}

impl From<&state::Robot> for Robot {
    fn from(robot: &state::Robot) -> Robot {
        Robot {
            id: robot.id,
            position: Some((&robot.position).into()),
            direction: EDirection::from(robot.direction).into(),
            damage: robot.damage,
            life_tokens: robot.life_tokens,
        }
    }
}

impl From<&state::Board> for Board {
    fn from(board: &state::Board) -> Board {
        let tiles: Vec<Tile> = board.tiles.iter()
            .map(Tile::from)
            .collect();
        Board {
            tiles,
            size_x: board.size_x,
            size_y: board.size_y,
        }
    }
}

impl From<&state::Tile> for Tile {
    fn from(tile: &state::Tile) -> Tile {
        let ttype: ETileType = tile.ttype.into();
        let walls = tile.walls.iter()
            .map(|dir| EDirection::from(*dir).into())
            .collect();
        Tile {
            position: Some((&tile.position).into()),
            r#type: ttype.into(),
            walls,
        }
    }
}

impl From<state::EDirection> for EDirection {
    fn from(dir: state::EDirection) -> EDirection {
        match dir {
            state::EDirection::NORTH => EDirection::North,
            state::EDirection::EAST => EDirection::East,
            state::EDirection::SOUTH => EDirection::South,
            state::EDirection::WEST => EDirection::West,
        }
    }
}

impl From<state::ETileType> for ETileType {
    fn from(ttype: state::ETileType) -> ETileType {
        match ttype {
            state::ETileType::Regular => ETileType::Regular,
            state::ETileType::NoTile => ETileType::NoTile,
        }
    }
}

impl From<&state::Position> for Position {
    fn from(pos: &state::Position) -> Position {
        Position {
            x: pos.x,
            y: pos.y,
        }
    }
}