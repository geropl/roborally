use tokio::{
    signal
};

extern crate anyhow;
use anyhow::{
    Result
};

#[macro_use] extern crate slog;

use singularlib as lib;
use lib::util::create_logger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log = create_logger();

    signal::ctrl_c().await?;
    info!(log, "sigint received, quitting.");

    Ok(())
}