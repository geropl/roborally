use std::path::PathBuf;

use tokio::signal;
use anyhow::Result;
#[macro_use] extern crate slog;

use singularlib as lib;
use lib::util;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = lib::Config::load_config(&PathBuf::from("config.json"))?;

    let log = util::create_logger();
    lib::do_run_singular(config, log.clone()).await?;

    signal::ctrl_c().await?;
    info!(log, "sigint received, quitting.");

    Ok(())
}