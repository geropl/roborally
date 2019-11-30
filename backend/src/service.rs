use tonic::{ Request, Response, Status, Code };
use failure::Error;

use std::sync::{ Arc, Mutex };

use crate::protocol::server::RoboRallyGame;
use crate::protocol::{ StartGameRequest, StartGameResponse, GetGameStateRequest, GetGameStateResponse, GameState, SetRoundInputRequest, SetRoundInputResponse };

use crate::game::state::{ State, Board, Player, RobotBuilder, Position, EDirection };
use crate::game::engine::move_engine::{ Engine };
use crate::game::engine::move_inputs::{ MoveInput, MoveInputs };

#[derive(Default)]
pub struct RoboRallyGameService {
    state: Arc<Mutex<State>>,
}

#[tonic::async_trait]
impl RoboRallyGame for RoboRallyGameService {
    async fn start_game(&self, _request: Request<StartGameRequest>) -> Result<Response<StartGameResponse>, Status> {
        let mut state = self.state.lock().unwrap();
        *state = new_game_state();

        let response = StartGameResponse{};
        Ok(Response::new(response))
    }

    async fn set_round_input(&self, request: Request<SetRoundInputRequest>) -> Result<Response<SetRoundInputResponse>, Status> {
        let game_state = match self.do_set_input(request.into_inner()) {
            Err(err) => {
                println!("Error: {}", err);
                return Err(Status::new(Code::Internal, format!("{}", err)))
            },
            Ok(s) => s,
        };

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
    fn do_set_input(&self, request: SetRoundInputRequest) -> Result<GameState, Error> {
        let move_input = MoveInput::parse_from(request.player_input)?;
        let move_ins = vec![move_input];
        let inputs = MoveInputs::from(move_ins.as_slice());

        let mut state = self.state.lock().unwrap();
        let current_state = (*state).clone();

        let engine = Engine::new();
        let new_state = engine.run_register_phase(Box::from(current_state), &inputs)?;

        let game_state = GameState::from(&new_state);
        *state = *new_state;

        Ok(game_state)
    }
}

fn new_game_state() -> State {
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

    let board = Board::new_empty_board(5, 5);
    State::new(board, vec![player1, player2])
}