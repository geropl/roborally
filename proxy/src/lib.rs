// use hyper::server::conn::AddrStream;
// use hyper::{Body, Request, Server};
// use hyper::service::{service_fn, make_service_fn};
use hyper::Client;

use futures::future;

use tokio::{
    task
};
use tokio::time::delay_for;

#[macro_use] extern crate anyhow;
use anyhow::{Result, Context};

#[macro_use] extern crate slog;

use kube::{
    config,
    client::APIClient
};

use std::time::Duration;

mod kubernetes;
use kubernetes::SingularEndpoint;

pub mod util;
use util::to_anyhow;

pub async fn do_run_singular(log: slog::Logger) -> Result<(), Box<dyn std::error::Error>> {
    info!(log, "Starting singular...");

    // Spawn task to 
    let log1 = log.clone();
    task::spawn(async move {
        if let Err(e) = run_endpoint_discovery(log1.clone()).await {
            error!(log1, "Error: {:?}", e);
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

async fn run_endpoint_discovery(log: slog::Logger) -> Result<()> {
    let config = config::load_kube_config().await
        .map_err(to_anyhow)
        .context("Failed loading kube config")?;
    let client = APIClient::new(config);

    info!(log, "singular running");

    let mut consecutive_err_count: u32 = 0;
    loop {
        debug!(log, "retrieving endpoint...");

        let res = get_singular_endpoint(log.clone(), client.clone()).await;
        if let Err(e) = res {
            consecutive_err_count += 1;
            if consecutive_err_count > 5 {
                return Err(e).context("Failed retrieving service endpoint");
            } else {
                warn!(log, "{}", e);
            }
        } else {
            consecutive_err_count = 0;
            // TODO notify proxy about new endpoint
        }
        
        delay_for(Duration::from_secs(3)).await;
    }
}


async fn get_singular_endpoint(log: slog::Logger, client: APIClient) -> anyhow::Result<SingularEndpoint> {
    let endpoints = kubernetes::get_service_endpoint(client).await?;

    // Query all endpoints if they think they are the one
    let singular_queries = endpoints.iter()
        .map(query_singular_endpoint);
    let singular_results = future::join_all(singular_queries).await;

    // Ignore failed queries
    let responses: Vec<&(SingularEndpoint, bool)> = singular_results.iter()
        .filter(|res| res.is_ok())
        .map(|res| res.as_ref().unwrap())
        .collect();
    
    // Sanity: Any responses at all?
    if responses.is_empty() {
        return Err(anyhow!("0/{} singular queries successful.", endpoints.len()));
    }

    // Filter by positive responses, e.g. which endpoints deemed themselves in charge
    let positive_response: Vec<&(SingularEndpoint, bool)> = responses.iter()
        .filter(|r| r.1)
        .cloned()
        .collect();
    match positive_response.len() {
        0 => Err(anyhow!("None of the singular endpoints felt responsible!")),
        1 => {
            let (endpoint, _) = positive_response.first().unwrap();
            Ok(endpoint.clone())
        },
        _ => {
            let all_endpoints_str = positive_response.iter()
                .map(|e| format!("{}", e.0))
                .collect::<Vec<String>>()
                .join(", ");
            warn!(log, "More than 1 singular endpoint felt responsible [{}], chosing first.", all_endpoints_str);

            let (endpoint, _) = positive_response.first().unwrap();
            Ok(endpoint.clone())
        }
    }
}

async fn query_singular_endpoint(endpoint: &SingularEndpoint) -> anyhow::Result<(SingularEndpoint, bool)> {
    let uri = format!("http://{}", endpoint).parse()?;
    let client = Client::new();
    let response = client.get(uri).await?;
    Ok((endpoint.clone(), response.status() == 200))
}