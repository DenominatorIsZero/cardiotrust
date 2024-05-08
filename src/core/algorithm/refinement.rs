use serde::{Deserialize, Serialize};
pub mod derivation;
pub mod update;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default, Copy)]
pub enum Optimizer {
    #[default]
    Sgd,
    Adam,
}
