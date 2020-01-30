
use std::time::Duration;

use hyper::Client;
use futures::future;
use tokio::time::delay_for;
use anyhow::{
    Result,
    Context
};
use slog::Logger;
use kube::{
    config,
    client::APIClient
};
use serde::{Deserialize};

use super::ServiceCoordinates;
use super::kubernetes;
use super::protocol::SingularEndpoint;

#[derive(Clone, Debug, Deserialize)]
pub struct DiscoveryOptions {
    pub interval: Duration,
    pub svc_coords: ServiceCoordinates,
    pub query_path: String,
}

pub async fn run_endpoint_discovery(opts: DiscoveryOptions, log: Logger) -> Result<()> {
    let config = config::load_kube_config().await
        .map_err(kubernetes::to_anyhow)
        .context("failed loading kube config")?;
    let client = APIClient::new(config);

    info!(log, "singular running");

    let mut consecutive_err_count: u32 = 0;
    loop {
        debug!(log, "retrieving endpoint...");

        let res = get_singular_endpoint(client.clone(), opts.clone(), log.clone()).await;
        if let Err(e) = res {
            consecutive_err_count += 1;
            if consecutive_err_count > 5 {
                return Err(e).context("failed retrieving singular endpoint");
            }

            warn!(log, "failed retrieving singular endpoint: {}", e);
        }

        consecutive_err_count = 0;
        // TODO notify proxy about new endpoint
        
        delay_for(opts.interval).await;
    }
}


async fn get_singular_endpoint(client: APIClient, opts: DiscoveryOptions, log: Logger) -> anyhow::Result<SingularEndpoint> {
    let endpoints = kubernetes::get_service_endpoint(client, &opts.svc_coords).await?;

    // Query all endpoints if they think they are the one
    let singular_queries = endpoints.iter()
        .map(|e| query_singular_endpoint(e, &opts.query_path));
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
        0 => {
            Err(anyhow!("none of the singular endpoints felt responsible!"))
        },
        1 => {
            let (endpoint, _) = positive_response.first().unwrap();
            Ok(endpoint.clone())
        },
        _ => {
            let all_endpoints_str = positive_response.iter()
                .map(|e| format!("{}", e.0))
                .collect::<Vec<String>>()
                .join(", ");
            warn!(log, "more than 1 singular endpoint felt responsible [{}], chosing first.", all_endpoints_str);

            let (endpoint, _) = positive_response.first().unwrap();
            Ok(endpoint.clone())
        }
    }
}

async fn query_singular_endpoint(endpoint: &SingularEndpoint, path: &str) -> anyhow::Result<(SingularEndpoint, bool)> {
    let uri = format!("http://{}{}", endpoint, path).parse()?;
    let client = Client::new();
    let response = client.get(uri).await?;
    Ok((endpoint.clone(), response.status() == 200))
}
