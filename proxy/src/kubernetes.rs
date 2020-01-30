use std::fmt;

use anyhow;
use kube::{
    api::{
        Api
    },
    client::APIClient,
};

use super::util::to_anyhow;

#[derive(Clone)]
pub struct SingularEndpoint {
    ip: String,
    port: i32,
}

impl fmt::Display for SingularEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.ip, self.port)
    }
}

pub async fn get_service_endpoint(client: APIClient) -> anyhow::Result<Vec<SingularEndpoint>> {
    let endpoints_api = Api::v1Endpoint(client).within("ingress");
    let endpoint = endpoints_api.get("staging-proxy")
        .await
        .map_err(to_anyhow)?;
    
    let port: i32 = 80;
    let mut result: Option<Vec<SingularEndpoint>> = None;
    for subset in endpoint.subsets {
        let (addrs, ports) = match (subset.addresses, subset.ports) {
            (Some(a), Some(p)) => (a, p),
            _ => continue,
        };
        if addrs.is_empty() {
            continue;
        }

        let port = match ports.iter().find(|&p| p.port == port) {
            Some(p) => p,
            None => continue,
        };

        let endpoints = addrs.iter()
            .map(|a| SingularEndpoint{
                ip: a.ip.clone(),
                port: port.port,
            })
            .collect();
        result.replace(endpoints);
        break;
    }
    result.ok_or_else(|| anyhow!("Could not find any Endpoint matching port and service name!"))
}