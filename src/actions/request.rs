use std::collections::HashMap;
// TODO: async use std::io::Read;

use colored::*;
use serde_json;
use std::iter;
use yaml_rust::Yaml;

use futures::{stream, Future, Stream};
use std::io::{self, Write};

use hyper::Client;
// use time;
// use hyper::Response;
// use crate::interpolator;

use crate::config;

use crate::actions::{Report, Runnable};

static USER_AGENT: &'static str = "drill";

#[derive(Clone)]
pub struct Request {
  name: String,
  url: String,
  time: f64,
  method: String,
  headers: HashMap<String, String>,
  pub body: Option<String>,
  pub with_item: Option<Yaml>,
  pub assign: Option<String>,
}

impl Request {
  pub fn is_that_you(item: &Yaml) -> bool {
    item["request"].as_hash().is_some()
  }

  pub fn new(item: &Yaml, with_item: Option<Yaml>) -> Request {
    let reference: Option<&str> = item["assign"].as_str();
    let body: Option<&str> = item["request"]["body"].as_str();
    let method = if let Some(v) = item["request"]["method"].as_str() {
      v.to_string().to_uppercase()
    } else {
      "GET".to_string()
    };

    let mut headers = HashMap::new();

    if let Some(hash) = item["request"]["headers"].as_hash() {
      for (key, val) in hash.iter() {
        if let Some(vs) = val.as_str() {
          headers.insert(key.as_str().unwrap().to_string(), vs.to_string());
        } else {
          panic!("{} Headers must be strings!!", "WARNING!".yellow().bold());
        }
      }
    }

    Request {
      name: item["name"].as_str().unwrap().to_string(),
      url: item["request"]["url"].as_str().unwrap().to_string(),
      time: 0.0,
      method: method,
      headers: headers,
      body: body.map(str::to_string),
      with_item: with_item,
      assign: reference.map(str::to_string),
    }
  }

  fn format_time(tdiff: f64, nanosec: bool) -> String {
    if nanosec {
      (1_000_000.0 * tdiff).round().to_string() + "ns"
    } else {
      tdiff.round().to_string() + "ms"
    }
  }
}

impl Runnable for Request {
  fn execute(&self, context: &mut HashMap<String, Yaml>, responses: &mut HashMap<String, serde_json::Value>, reports: &mut Vec<Report>, config: &config::Config) {
    if self.with_item.is_some() {
      context.insert("item".to_string(), self.with_item.clone().unwrap());
    }

    // tokio::run(move || {
    //   let client = Client::new();
    //   let uri = "http://localhost:9000/api/users.json".parse().unwrap();

    //   client
    //     .get(uri)
    //     .map(move |_res| {})
    //     .map_err(|err| {
    //       println!("Error: {}", err);
    //     })
    // });
  }

  fn has_interpolations(&self) -> bool {
    self.name.contains("{") ||
      self.url.contains("{") ||
      self.body.clone().unwrap_or("".to_string()).contains("{") ||
      self.with_item.is_some() ||
      self.assign.is_some() ||
      false // TODO: headers
  }

  fn extreme(&self, iterations: usize) {
    let absolute_url = format!("http://localhost:9000{}", self.url);
    let client = Client::new();
    let uri = absolute_url.parse().unwrap();
    let uris = iter::repeat(uri).take(iterations);

    let work = stream::iter_ok(uris)
        .map(move |uri| client.get(uri))
        .buffer_unordered(250)
        .and_then(|res| {
            println!("Response: {}", res.status());
            res.into_body()
                .concat2()
                .map_err(|e| panic!("Error collecting body: {}", e))
        })
        .for_each(|body| {
            io::stdout()
                .write_all(&body)
                .map_err(|e| panic!("Error writing: {}", e))
        })
        .map_err(|e| panic!("Error making request: {}", e));

    tokio::run(work);
  }
}
