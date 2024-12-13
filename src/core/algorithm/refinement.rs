use std::fmt::Display;

use serde::{Deserialize, Serialize};
pub mod derivation;
pub mod update;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default, Copy)]
pub enum Optimizer {
    #[default]
    Sgd,
    Adam,
}

impl Display for Optimizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sgd => write!(f, "SGD"),
            Self::Adam => write!(f, "Adam"),
        }
    }
}
