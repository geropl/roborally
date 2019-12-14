use tonic::{ Request, Response, Status, Code };
use failure::Error;

use std::sync::{ Arc, Mutex };

use crate::protocol::server::RoboRallyGame;
use crate::protocol::{ StartGameRequest, StartGameResponse, GetGameStateRequest, GetGameStateResponse, GameState, SetRoundInputRequest, SetRoundInputResponse };

use crate::roborally::state::{ State, Board, Player, RobotBuilder, Position, EDirection, RoundPhase, ParserError as BoardParserError };
use crate::roborally::engine::round_engine::{ RoundEngine };
use crate::roborally::engine::player_input::{ PlayerInput };

#[derive(Default)]
pub struct RoboRallyGameService {
    state: Arc<Mutex<State>>,
}

#[tonic::async_trait]
impl RoboRallyGame for RoboRallyGameService {
    async fn start_game(&self, _request: Request<StartGameRequest>) -> Result<Response<StartGameResponse>, Status> {
        let game_state = self.start_new_game().map_err(into_status)?;

        Ok(Response::new(StartGameResponse{
            state: Some(game_state),
        }))
    }

    async fn set_round_input(&self, request: Request<SetRoundInputRequest>) -> Result<Response<SetRoundInputResponse>, Status> {
        let game_state = self.do_set_input(request.into_inner()).map_err(into_status)?;

        let response = SetRoundInputResponse{
            state: Some(game_state),
        };
        Ok(Response::new(response))
    }

    async fn get_game_state(&self, _request: Request<GetGameStateRequest>) -> Result<Response<GetGameStateResponse>, Status> {
        let state = self.state.lock().unwrap();
        let response = GetGameStateResponse {
            state: Some(GameState::from(&*state)),
        };
        Ok(Response::new(response))
    }
}

impl RoboRallyGameService {
    fn start_new_game(&self) -> Result<GameState, Error> {
        let mut state = new_game_state()?;
        let engine = RoundEngine::new();
        state = engine.run_round_initialization(state)?;
        
        let game_state = GameState::from(&state);
        let mut persistent_state = self.state.lock().unwrap();
        *persistent_state = *state;

        Ok(game_state)
    }

    fn do_set_input(&self, request: SetRoundInputRequest) -> Result<GameState, Error> {
        let player_input = PlayerInput::parse_from(request.player_input)?;

        let mut persistent_state = self.state.lock().unwrap();
        let mut state = Box::from((*persistent_state).clone());

        let engine = RoundEngine::new();
        state = engine.set_player_input(state, &player_input)?;

        if state.phase == RoundPhase::EXECUTE {
            state = engine.run_execute(state)?;
        }

        let game_state = GameState::from(&state);
        *persistent_state = *state;

        Ok(game_state)
    }
}

fn new_game_state() -> Result<Box<State>, BoardParserError> {
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

    let board = Board::load_board("test1")?;
    Ok(State::new_with_random_deck(board, vec![player1, player2]))
}

fn into_status(err: Error) -> Status {
    Status::new(Code::Internal, format!("{}", err))
}