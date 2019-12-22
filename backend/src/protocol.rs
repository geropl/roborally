use failure::Fail;

use crate::roborally::state;
use crate::roborally::engine::player_input;
use crate::roborally::engine::register_engine;

tonic::include_proto!("protocol");

// Protocol -> state
#[derive(Debug, Fail)]
pub enum ProtocolError {
    #[fail(display = "Missing player input!")]
    MissingPlayerInput {
    },
}

impl player_input::ProgramInput {
    pub fn parse_from(player_input: Option<ProgramInput>) -> Result<player_input::ProgramInput, ProtocolError> {
        let player_input = player_input.ok_or(ProtocolError::MissingPlayerInput{})?;

        Ok(player_input::ProgramInput {
            player_id: player_input.player_id,
            register_cards_choices: player_input.register_cards_choices, 
        })
    }
}

impl player_input::StartPositionInput {
    pub fn parse_from(player_input: Option<StartPositionInput>) -> Result<player_input::StartPositionInput, ProtocolError> {
        let player_input = player_input.ok_or(ProtocolError::MissingPlayerInput{})?;

        Ok(player_input::StartPositionInput {
            player_id: player_input.player_id,
            start_position_id: player_input.start_position_id, 
        })
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
impl From<&state::GameState> for GameState {
    fn from(game_state: &state::GameState) -> GameState {
        use std::borrow::Borrow;
        GameState {
            phase: EGamePhase::from(game_state.phase).into(),
            initial_state: Some(State::from(game_state.initial_state())),
            start_state: Some(State::from(game_state.start_state.borrow())),
            rounds: game_state.all_rounds().map(Round::from).collect(),
            game_result: from_game_result(&game_state.game_result),
        }
    }
}

impl From<state::EGamePhase> for EGamePhase {
    fn from(phase: state::EGamePhase) -> EGamePhase {
        match phase {
            state::EGamePhase::INITIAL => EGamePhase::Initial,
            state::EGamePhase::PREPARATION => EGamePhase::Preparation,
            state::EGamePhase::RUNNING => EGamePhase::Running,
            state::EGamePhase::ENDED => EGamePhase::Ended,
        }
    }
}

pub fn from_game_result(result: &state::EGameResult) -> Option<game_state::GameResult> {
    match result {
        state::EGameResult::None => None,
        state::EGameResult::Draw{ player_ids } => Some(game_state::GameResult::Draw(GameResultDraw{ player_ids: player_ids.clone() })),
        state::EGameResult::Win{ player_id } => Some(game_state::GameResult::Win(GameResultWin{ player_id: *player_id })),
    }
}

impl From<&state::Round> for Round {
    fn from(round: &state::Round) -> Round {
        use std::borrow::Borrow;
        Round {
            id: round.id,
            phase: ERoundPhase::from(round.phase).into(),
            state: Some(State::from(round.state.borrow())),
        }
    }
}

impl From<state::ERoundPhase> for ERoundPhase {
    fn from(phase: state::ERoundPhase) -> ERoundPhase {
        match phase {
            state::ERoundPhase::INITIALIZATION => ERoundPhase::Initialization,
            state::ERoundPhase::PROGRAMMING => ERoundPhase::Programming,
            state::ERoundPhase::EXECUTION => ERoundPhase::Execution,
            state::ERoundPhase::CLEANUP => ERoundPhase::Cleanup,
            state::ERoundPhase::DONE => ERoundPhase::Done,
        }
    }
}

impl From<&state::State> for State {
    fn from(state: &state::State) -> State {
        use std::borrow::Borrow;

        let players: Vec<Player> = state.all_players()
            .map(Player::from)
            .collect();
        let cards = state.deck.cards.iter()
            .map(MoveCard::from)
            .collect();
        State {
            board: Some(Board::from(state.board.borrow())),
            players,
            cards,
        }
    }
}

impl From<&state::Player> for Player {
    fn from(player: &state::Player) -> Player {
        let program_card_deck = player.program_card_deck.iter()
            .map(MoveCard::from)
            .collect();
        let registers = player.registers.iter()
            .map(Register::from)
            .collect();
        Player {
            id: player.id,
            robot: Some(Robot::from(&player.robot)),
            registers,
            program_card_deck,
            input_required: player.input_required,
        }
    }
}

impl From<&state::Register> for Register {
    fn from(register: &state::Register) -> Register {
        Register {
            move_card: register.move_card.as_ref().map(MoveCard::from),
            locked: register.locked,
        }
    }
}

impl From<&state::MoveCard> for MoveCard {
    fn from(card: &state::MoveCard) -> MoveCard {
        let moves = card.tmove.iter()
            .cloned()
            .map(ESimpleMove::from)
            .map(|m| m as i32)
            .collect();
        MoveCard {
            id: card.id,
            priority: card.priority,
            moves,
        }
    }
}

impl From<register_engine::ESimpleMove> for ESimpleMove {
    fn from(dir: register_engine::ESimpleMove) -> ESimpleMove {
        match dir {
            register_engine::ESimpleMove::Backward => ESimpleMove::Backward,
            register_engine::ESimpleMove::Forward => ESimpleMove::Forward,
            register_engine::ESimpleMove::StepLeft => ESimpleMove::StepLeft,
            register_engine::ESimpleMove::StepRight => ESimpleMove::StepRight,
            register_engine::ESimpleMove::TurnLeft => ESimpleMove::TurnLeft,
            register_engine::ESimpleMove::TurnRight => ESimpleMove::TurnRight,
            register_engine::ESimpleMove::UTurn => ESimpleMove::UTurn,
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
            start_position_id: match tile.start_position_id {
                None => None,
                Some(id) => Some(StartPositionId{ id }),
            },
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