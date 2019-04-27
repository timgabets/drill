mod assign;
mod request;

pub use self::assign::Assign;
pub use self::request::Request;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::fmt;
use yaml_rust::Yaml;
use futures::Future;

use crate::config;

pub trait Runnable {
  fn execute(
      &self,
      context: &Arc<Mutex<HashMap<String, Yaml>>>,
      responses: &Arc<Mutex<HashMap<String, serde_json::Value>>>,
      reports: &Arc<Mutex<Vec<Report>>>,
      config: &config::Config
  ) -> (
    Box<Future<Item=(), Error=()> + Send>
  );
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
