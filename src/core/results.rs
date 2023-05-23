use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Results {
    metrics: Metrics,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Metrics;
