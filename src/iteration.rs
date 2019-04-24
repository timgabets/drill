use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

use yaml_rust::Yaml;
use futures::Future;

use crate::actions::{Report, Runnable};

#[derive(Clone)]
pub struct Iteration {
  pub number: i64,
  pub context: HashMap<String, Yaml>,
  pub responses: HashMap<String, serde_json::Value>,
  pub reports: Vec<Report>
}

impl Iteration {
  pub fn future(&self, benchmark: &Arc<Vec<Box<(Runnable + Sync + Send)>>>) -> bool {
    true
  }
}
