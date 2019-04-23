mod assign;
mod request;

pub use self::assign::Assign;
pub use self::request::Request;
use crate::config;

use std::collections::HashMap;
use std::fmt;

use serde_json::Value;
use yaml_rust::Yaml;
use futures::Future;

pub trait Runnable {
  fn execute<'a>(&'a self, context: &'a mut HashMap<String, Yaml>, responses: &'a mut HashMap<String, serde_json::Value>, reports: &'a mut Vec<Report>, config: &'a config::Config) -> Box<Future<Item=(), Error=()> + Send + 'a>;
  fn has_interpolations(&self) -> bool;
}

#[derive(Clone)]
pub struct Report {
  pub name: String,
  pub duration: f64,
  pub status: u16,
}

impl fmt::Debug for Report {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "\n- name: {}\n  duration: {}\n", self.name, self.duration)
  }
}

impl fmt::Display for Report {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "\n- name: {}\n  duration: {}\n  status: {}\n", self.name, self.duration, self.status)
  }
}
