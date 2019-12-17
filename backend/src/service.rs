use tonic::{ Request, Response, Status, Code };
use failure::Error;

use std::sync::{ Arc, Mutex };

use crate::protocol::server::RoboRallyGame;
use crate::protocol::{ StartGameRequest, StartGameResponse, GetGameStateRequest, GetGameStateResponse, GameState, SetProgramInputRequest, SetProgramInputResponse };

use crate::roborally::state as s;
use crate::roborally::engine::game_engine::{ GameEngine };
use crate::roborally::engine::player_input::{ PlayerInput };

#[derive(Default)]
pub struct RoboRallyGameService {
    state: Arc<Mutex<s::GameState>>,
}

#[tonic::async_trait]
impl RoboRallyGame for RoboRallyGameService {
    async fn start_game(&self, _request: Request<StartGameRequest>) -> Result<Response<StartGameResponse>, Status> {
        let game_state = self.start_new_game().map_err(into_status)?;

        Ok(Response::new(StartGameResponse{
            state: Some(game_state),
        }))
    }

    async fn set_program_input(&self, request: Request<SetProgramInputRequest>) -> Result<Response<SetProgramInputResponse>, Status> {
        let game_state = self.do_set_program_input(request.into_inner()).map_err(into_status)?;

        let response = SetProgramInputResponse{
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
        let mut game_state = new_game_state()?;
        let engine = GameEngine::new();
        engine.initialize(&mut game_state)?;
        
        let proto_game_state = GameState::from(&game_state);
        let mut persistent_state = self.state.lock().unwrap();
        *persistent_state = game_state;

        Ok(proto_game_state)
    }

    fn do_set_program_input(&self, request: SetProgramInputRequest) -> Result<GameState, Error> {
        let player_input = PlayerInput::parse_from(request.player_input)?;

        let mut persistent_state = self.state.lock().unwrap();
        let mut game_state = (*persistent_state).clone();

        let engine = GameEngine::new();
        engine.set_player_program_input(&mut game_state, &player_input)?;

        let proto_game_state = GameState::from(&game_state);
        *persistent_state = game_state;

        Ok(proto_game_state)
    }
}

fn new_game_state() -> Result<s::GameState, Error> {
    use s::*;

    let config = GameConfig::default();
    let game_state = GameState::create_from(&config)?;
    Ok(game_state)
}

fn into_status(err: Error) -> Status {
    Status::new(Code::Internal, format!("{}", err))
}