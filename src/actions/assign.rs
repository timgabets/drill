use colored::*;
use futures::Future;
use yaml_rust::Yaml;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::config;
use crate::interpolator::Interpolator;
use crate::actions::{Report, Runnable};

#[derive(Clone)]
pub struct Assign {
  name: String,
  key: String,
  value: String,
}

impl Assign {
  pub fn is_that_you(item: &Yaml) -> bool {
    item["assign"].as_hash().is_some()
  }

  pub fn new(item: &Yaml, _with_item: Option<Yaml>) -> Assign {
    Assign {
      name: item["name"].as_str().unwrap().to_string(),
      key: item["assign"]["key"].as_str().unwrap().to_string(),
      value: item["assign"]["value"].as_str().unwrap().to_string(),
    }
  }
}

impl Runnable for Assign {
  fn execute<'a>(
      &'a self,
      context: &'a Arc<Mutex<HashMap<String, Yaml>>>,
      responses: &'a Arc<Mutex<HashMap<String, serde_json::Value>>>,
      reports: &'a Arc<Mutex<Vec<Report>>>,
      config: &'a config::Config
  ) -> (
    Box<Future<Item=(), Error=()> + Send + 'a>
  ) {
    let mut context = context.lock().unwrap();
    let mut responses = responses.lock().unwrap();
    let mut reports = reports.lock().unwrap();

    if !config.quiet {
      println!("{:width$} {}={}", self.name.green(), self.key.cyan().bold(), self.value.magenta(), width = 25);
    }
    // TODO: Should we interpolate the value?
    context.insert(self.key.to_owned(), Yaml::String(self.value.to_owned()));

    Box::new(futures::future::ok(()))
  }

  fn has_interpolations(&self) -> bool {
    Interpolator::has_interpolations(&self.name) ||
    Interpolator::has_interpolations(&self.value)
  }
}
