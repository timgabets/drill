use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use yaml_rust::Yaml;
use futures::Future;

use crate::actions::{Report, Runnable};
use crate::config;

#[derive(Clone)]
pub struct Iteration {
  pub number: i64,
  pub context: Arc<Mutex<HashMap<String, Yaml>>>,
  pub responses: Arc<Mutex<HashMap<String, serde_json::Value>>>,
  pub reports: Arc<Mutex<Vec<Report>>>,
}

impl Iteration {
  pub fn future<'a>(
    &'a self,
    benchmark: &'a Arc<Vec<Box<(Runnable + Sync + Send)>>>,
    config: &'a config::Config
  ) -> Box<Future<Item=(), Error=()> + Send + 'a> {
    let all = benchmark.iter().map(move |item| {
      item.execute(&self.context, &self.responses, &self.reports, config)
    });

    // FIXME
    let work = futures::future::join_all(all)
      .map(|_e| ())
      .map_err(|_err| ());
    // let work = all.nth(0).unwrap();

    Box::new(work)
  }
}
