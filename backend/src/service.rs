use tonic::{ Request, Response, Status, Code };

use std::sync::{ Arc, Mutex };

use crate::protocol::server::RoboRallyGame;
use crate::protocol::{ StartGameRequest, StartGameResponse, GetGameStateRequest, GetGameStateResponse, GameState };

use crate::game::state::{ State, Board, Player, RobotBuilder, Position, EDirection };
use crate::game::engine::{ Engine, SimpleMove, ESimpleMove, MoveCard, MoveInputBuilder, MoveInputs };

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

    async fn get_game_state(&self, request: Request<GetGameStateRequest>) -> Result<Response<GetGameStateResponse>, Status> {
        println!("Got a request: {:?}", request);

        let mut state = self.state.lock().unwrap();

        // TODO Make Inputs inputable
        let move_forward = SimpleMove::single(ESimpleMove::Forward);
        let move_card1 = MoveCard::new(1, move_forward);
        let move_input1 = MoveInputBuilder::default()
            .player_id(0)
            .mmove(move_card1)
            .build().unwrap();

        let move_left_forward = SimpleMove::new(&[ESimpleMove::TurnLeft, ESimpleMove::Forward]);
        let move_card2 = MoveCard::new(2, move_left_forward);
        let move_input2 = MoveInputBuilder::default()
            .player_id(1)
            .mmove(move_card2)
            .build().unwrap();

        let ins = vec![move_input1, move_input2];
        let inputs = MoveInputs::from(ins.as_slice());

        let engine = Engine::new();
        let current_state = (*state).clone();
        let new_state = engine.run_register_phase(Box::from(current_state), &inputs);
        if let Err(err) = new_state {
            println!("Error: {}", err);
            return Err(Status::new(Code::Internal, format!("{}", err)))
        }
        let new_state = new_state.unwrap();

        let response = GetGameStateResponse {
            state: Some(GameState::from(&new_state)),
        };
        *state = *new_state;

        Ok(Response::new(response))
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