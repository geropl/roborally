use failure::Fail;

use crate::game::state;
use crate::game::engine::move_inputs;
use crate::game::engine::move_engine;

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

impl move_inputs::MoveInput {
    pub fn parse_from(player_input: Option<PlayerInput>) -> Result<move_inputs::MoveInput, ProtocolError> {
        if player_input.is_none() {
            return Err(ProtocolError::MissingPlayerInput{});
        }
        let player_input = player_input.unwrap();

        let move_cards: Result<Vec<move_inputs::MoveCard>, _> = player_input.move_cards.iter()
            .map(move_inputs::MoveCard::parse_from)
            .collect();
        Ok(move_inputs::MoveInput {
            player_id: player_input.player_id,
            move_cards: move_cards?, 
        })
    }
}

impl move_inputs::MoveCard {
    fn parse_from(move_card: &MoveCard) -> Result<move_inputs::MoveCard, ProtocolError> {
        let simple_moves: Result<Vec<move_engine::ESimpleMove>, _> = move_card.moves.iter()
            .map(|mmove_i32| move_engine::ESimpleMove::parse_from(*mmove_i32))
            .collect();
        let simple_moves = simple_moves?;
        Ok(move_inputs::MoveCard {
            priority: move_card.priority,
            tmove: move_inputs::SimpleMove::new(&simple_moves),
        })
    }
}

impl move_engine::ESimpleMove {
    fn parse_from(mmove_i32: i32) -> Result<move_engine::ESimpleMove, ProtocolError> {
        let mmove = match ESimpleMove::from_i32(mmove_i32) {
            None => {
                return Err(ProtocolError::WrongEnumValue{
                    enum_name: String::from("ESimpleMove"),
                    value: mmove_i32,
                });
            },
            Some(m) => m,
        };

        Ok(move_engine::ESimpleMove::from(mmove))
    }
}

impl From<ESimpleMove> for move_engine::ESimpleMove {
    fn from(mmove: ESimpleMove) -> move_engine::ESimpleMove {
        match mmove {
            ESimpleMove::Forward => move_engine::ESimpleMove::Forward,
            ESimpleMove::Backward => move_engine::ESimpleMove::Backward,
            ESimpleMove::StepLeft => move_engine::ESimpleMove::StepLeft,
            ESimpleMove::StepRight => move_engine::ESimpleMove::StepRight,
            
            ESimpleMove::TurnRight => move_engine::ESimpleMove::TurnRight,
            ESimpleMove::TurnLeft => move_engine::ESimpleMove::TurnLeft,
            ESimpleMove::UTurn => move_engine::ESimpleMove::UTurn,
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
            state::ETileType::Free => ETileType::Free,
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