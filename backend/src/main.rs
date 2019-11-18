use tonic::{transport::Server, Request, Response, Status};

use std::env;

mod protocol;
use protocol::server::{ RoboRallyGame, RoboRallyGameServer };
use protocol::{ GetGameStateRequest, GetGameStateResponse, GameState, Board };

mod game;

#[derive(Default)]
pub struct RoboRallyGameService {}

#[tonic::async_trait]
impl RoboRallyGame for RoboRallyGameService {
    async fn get_game_state(&self, request: Request<GetGameStateRequest>) -> Result<Response<GetGameStateResponse>, Status> {
        println!("Got a request: {:?}", request);

        let response = GetGameStateResponse {
            state: Some(GameState {
                board: Some(Board {
                    size_x: 0,
                    size_y: 0,
                    tiles: vec![],
                }),
                players: vec![],
            }),
        };
        Ok(Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let socket_addr = match &args.as_slice()[1..] {
        [socket_addr_str] => {
            socket_addr_str.parse().unwrap()
        },
        _ => {
            panic!("Expected arguments: <address>:<port>!")
        }
    };

    let service = RoboRallyGameService::default();
    Server::builder()
        .serve(socket_addr, RoboRallyGameServer::new(service))
        .await?;

    Ok(())
}