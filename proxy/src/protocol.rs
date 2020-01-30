use std::fmt;

use serde::{Deserialize};

#[derive(Clone, Debug, Deserialize)]
pub struct ServiceCoordinates {
    pub name: String,
    pub namespace: String,
    pub port: i32,
}

#[derive(Clone, Debug)]
pub struct SingularEndpoint {
    pub ip: String,
    pub port: i32,
}

impl fmt::Display for SingularEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.ip, self.port)
    }
}
