use std::path::PathBuf;

use crate::event::Detection;

/// A removal request is a plan only. This task does not unlink paths.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemovalPlan {
    pub paths: Vec<PathBuf>,
    pub executed: bool,
}

pub struct RemoveTask;

impl RemoveTask {
    pub fn plan(detections: &[Detection]) -> RemovalPlan {
        RemovalPlan {
            paths: detections.iter().map(|d| d.path.clone()).collect(),
            executed: false,
        }
    }
}
