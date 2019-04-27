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
    let mut all = benchmark.iter().map(|item| {
      let context = self.context.clone();
      let responses = self.responses.clone();
      let reports = self.reports.clone();

      item.execute(&context, &responses, &reports, config)
    });

    // FIXME
    // let work = futures::future::join_all(all);
    let work = all.nth(0).unwrap();

    Box::new(work)
  }
}
