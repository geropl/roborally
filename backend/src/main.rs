use tonic::{transport::Server, Request, Response, Status};

use std::env;

mod protocol;
use protocol::{
    server::{Greeter, GreeterServer},
    HelloReply, HelloRequest,
};

mod game;

#[derive(Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello( &self, request: Request<HelloRequest>) -> Result<Response<HelloReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = protocol::HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };
        Ok(Response::new(reply))
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
    let greeter = MyGreeter::default();

    Server::builder()
        .serve(socket_addr, GreeterServer::new(greeter))
        .await?;

    Ok(())
}