mod protocol;
mod service;
mod roborally;

use tonic::transport::Server;

use std::env;

use protocol::robo_rally_game_server::RoboRallyGameServer;

use service::RoboRallyGameService;

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
        .add_service(RoboRallyGameServer::new(service))
        .serve(socket_addr)
        .await?;

    Ok(())
}