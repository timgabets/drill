use std::collections::HashMap;

use colored::*;
use futures::Future;
use futures::future::ok;
use yaml_rust::Yaml;

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
      context: &'a mut HashMap<String, Yaml>,
      responses: &'a mut HashMap<String, serde_json::Value>,
      reports: &'a mut Vec<Report>,
      config: &'a config::Config
  ) -> (
    Box<
      Future<Item=(
        &mut HashMap<String, Yaml>,
        &mut HashMap<String, serde_json::Value>,
        &mut Vec<Report>
      ), Error=()>
    + Send + 'a>
  ) {
    if !config.quiet {
      println!("{:width$} {}={}", self.name.green(), self.key.cyan().bold(), self.value.magenta(), width = 25);
    }
    // TODO: Should we interpolate the value?
    context.insert(self.key.to_owned(), Yaml::String(self.value.to_owned()));

    // TODO: Create a future here
    (Box::new(ok((context, responses, reports))))
  }

  fn has_interpolations(&self) -> bool {
    Interpolator::has_interpolations(&self.name) ||
    Interpolator::has_interpolations(&self.value)
  }
}
