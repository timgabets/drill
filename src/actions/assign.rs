use std::collections::HashMap;

use colored::*;
use serde_json::Value;
use yaml_rust::Yaml;

use crate::config;
use crate::interpolator::Interpolator;
use crate::actions::{Report, Runnable};
use futures::Future;
use futures::future::ok;

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
  fn execute(&self, context: &mut HashMap<String, Yaml>, _responses: &mut HashMap<String, Value>, _reports: &mut Vec<Report>, _config: &config::Config) {
    if !_config.quiet {
      println!("{:width$} {}={}", self.name.green(), self.key.cyan().bold(), self.value.magenta(), width = 25);
    }
    // TODO: Should we interpolate the value?
    context.insert(self.key.to_owned(), Yaml::String(self.value.to_owned()));
  }

  fn has_interpolations(&self) -> bool {
    Interpolator::has_interpolations(&self.name) ||
    Interpolator::has_interpolations(&self.value)
  }

  fn extreme(&self, _iterations: usize) {
    // Do nothing
  }

  fn async_execute(&self) -> Box<Future<Item=(), Error=()> + Send> {
    Box::new(ok(()))
  }
}
