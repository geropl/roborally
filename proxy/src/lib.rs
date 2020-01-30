
mod protocol;
mod kubernetes;
mod discovery;
pub mod util;

use std::path::Path;
use std::fs;
use tokio::task;
#[macro_use] extern crate anyhow;
use anyhow::Result;
#[macro_use] extern crate slog;
use slog::Logger;
use serde::{Deserialize};
use serde_json;

use protocol::ServiceCoordinates;
use discovery::{
    DiscoveryOptions,
    run_endpoint_discovery
};

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub discovery: DiscoveryOptions,
}
impl Config {
    pub fn load_config(path: &Path) -> Result<Config> {
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content)
            .map_err(|e| anyhow!("{}", e))
    }
}

pub async fn do_run_singular(config: Config, log: Logger) -> Result<(), Box<dyn std::error::Error>> {
    info!(log, "starting singular...");

    // Spawn task to 
    let log1 = log.clone();
    task::spawn(async move {
        if let Err(e) = run_endpoint_discovery(config.discovery, log1.clone()).await {
            error!(log1, "error: {:?}", e);
        }
    });


    // // This is our socket address...
    // let addr = ([127, 0, 0, 1], 8081).into();

    // // A `Service` is needed for every connection.
    // let make_svc = make_service_fn(|socket: &AddrStream| {
    //     let remote_addr = socket.remote_addr();
    //     service_fn(move |req: Request<Body>| { // returns BoxFut
    //         hyper_reverse_proxy::call(remote_addr.ip(), "http://127.0.0.1:8080", req)
    //     })
    // });

    // let server = Server::bind(&addr)
    //     .serve(make_svc)
    //     .map_err(|e| eprintln!("server error: {}", e));

    // println!("Running server on {:?}", addr);

    // Run this server for... forever!
    // hyper::rt::run(server);

    Ok(())
}
