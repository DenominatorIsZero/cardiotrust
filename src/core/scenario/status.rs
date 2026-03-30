use serde::{Deserialize, Serialize};

/// Enumeration of possible scenario execution statuses.
///
/// * `Planning`: Scenario is being planned.
/// * `Done`: Scenario execution finished.
/// * `Running`: Scenario is running the specified epoch.
/// * `Aborted`: Scenario execution was aborted.
/// * `Scheduled`: Scenario execution is scheduled but not yet running.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum Status {
    Planning,
    Done,
    Simulating,
    Running(usize),
    Aborted,
    Scheduled,
}
