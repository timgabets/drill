use colored::*;
use futures::{stream, Future, Stream};
use hyper::{Client, Response};
use hyper_tls::HttpsConnector;
use serde_json;
use std::collections::HashMap;
use std::io::{self, Write};
use std::iter;
use time;
use yaml_rust::Yaml;

use crate::actions::{Report, Runnable};
use crate::config;
use crate::interpolator::Interpolator;

static USER_AGENT: &'static str = "drill";
static CONCURRENCY: usize = 250;

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

  fn send_request(&self, context: &mut HashMap<String, Yaml>, responses: &mut HashMap<String, serde_json::Value>, reports: &mut Vec<Report>, config: &config::Config) {
    if self.with_item.is_some() {
      context.insert("item".to_string(), self.with_item.clone().unwrap());
    }

    let begin = time::precise_time_s();
    let mut uninterpolator = None;

    // Resolve the name
    let interpolated_name = if self.name.contains("{") {
      uninterpolator.get_or_insert(Interpolator::new(context, responses)).resolve(&self.name)
    } else {
      self.name.clone()
    };

    // Resolve the url
    let interpolated_url = if self.url.contains("{") {
      uninterpolator.get_or_insert(Interpolator::new(context, responses)).resolve(&self.url)
    } else {
      self.url.clone()
    };

    // Resolve relative urls
    let interpolated_base_url = if &interpolated_url[..1] == "/" {
      match context.get("base") {
        Some(value) => {
          if let Some(vs) = value.as_str() {
            format!("{}{}", vs.to_string(), interpolated_url)
          } else {
            panic!("{} Wrong type 'base' variable!", "WARNING!".yellow().bold());
          }
        }
        _ => {
          panic!("{} Unknown 'base' variable!", "WARNING!".yellow().bold());
        }
      }
    } else {
      interpolated_url
    };

    let client = if interpolated_base_url.starts_with("https") {
      // Build a TSL connector
      // TODO
      // let mut connector_builder = TlsConnector::builder();
      // connector_builder.danger_accept_invalid_certs(config.no_check_certificate);

      // let ssl = NativeTlsClient::from(connector_builder.build().unwrap());
      // let connector = HttpsConnector::new(ssl);

      // Client::with_connector(connector)

      let https = HttpsConnector::new(4).expect("TLS initialization failed");
      Client::builder().build::<_, hyper::Body>(https)
    } else {
      Client::new();

      // FIXME
      let https = HttpsConnector::new(4).expect("TLS initialization failed");
      Client::builder().build::<_, hyper::Body>(https)
    };

    let interpolated_body;

    // Resolve the body
    let mut request = if let Some(body) = self.body.as_ref() {
      interpolated_body = uninterpolator.get_or_insert(Interpolator::new(context, responses)).resolve(body);

      // client.request(method, interpolated_base_url.as_str()).body(&interpolated_body)

      hyper::Request::builder()
        .method(self.method.to_uppercase().as_str())
        .uri(interpolated_base_url)
        .body(hyper::Body::from(interpolated_body))
        .expect("request builder with body")
    } else {
      // client.request(method, interpolated_base_url.as_str())
      hyper::Request::builder()
        .method(self.method.to_uppercase().as_str())
        .uri(interpolated_base_url)
        .body(hyper::Body::from(""))
        .expect("request builder without body")
    };

    // Headers
    let headers = request.headers_mut();
    headers.insert(hyper::header::USER_AGENT, USER_AGENT.parse().unwrap());

    if let Some(cookie) = context.get("cookie") {
      headers.insert(hyper::header::COOKIE, cookie.as_str().unwrap().parse().unwrap());
    }

    // Resolve headers
    for (key, val) in self.headers.iter() {
      let interpolated_header = uninterpolator.get_or_insert(Interpolator::new(context, responses)).resolve(val);

      let header_name = hyper::header::HeaderName::from_lowercase(key.to_lowercase().as_bytes()).unwrap();
      headers.insert(header_name, interpolated_header.parse().unwrap());
    }

    client
      .request(request)
      .map(move |response| {
        let duration_ms = (time::precise_time_s() - begin) * 1000.0;

        if !config.quiet {
          let message = response.status().to_string();
          let _status_text = if response.status().is_server_error() {
            message.red()
          } else if response.status().is_client_error() {
            message.purple()
          } else {
            message.yellow()
          };

          // TODO: println!("{:width$} {} {} {}", interpolated_name.green(), interpolated_base_url.blue().bold(), status_text, Request::format_time(duration_ms, config.nanosec).cyan(), width = 25);
        }

        reports.push(Report {
          name: self.name.to_owned(),
          duration: duration_ms,
          status: response.status().as_u16(),
        });

        if let Some(cookie) = response.headers().get(hyper::header::SET_COOKIE) {
          let value = String::from(cookie.to_str().unwrap().split(";").next().unwrap());
          context.insert("cookie".to_string(), Yaml::String(value));
        }

        if let Some(ref key) = self.assign {
          // let data = String::new();

          // TODO:
          // response.read_to_string(&mut data).unwrap();
          let data = "YOLO";

          let value: serde_json::Value = serde_json::from_str(&data).unwrap();

          responses.insert(key.to_owned(), value);
        }
      })
      .map_err(|_err| {
        let _duration_ms = (time::precise_time_s() - begin) * 1000.0;

        if !config.quiet {
          // TODO: println!("Error connecting '{}': {:?}", &interpolated_base_url.clone().as_str(), err);
        }

        // TODO
        // reports.push(Report {
        //   name: self.name.to_owned(),
        //   duration: duration_ms,
        //   status: 520u16,
        // });
      });
  }
}

impl Runnable for Request {
  fn execute(&self, context: &mut HashMap<String, Yaml>, responses: &mut HashMap<String, serde_json::Value>, reports: &mut Vec<Report>, config: &config::Config) {
    self.send_request(context, responses, reports, config);
  }

  fn has_interpolations(&self) -> bool {
    Interpolator::has_interpolations(&self.name) ||
    Interpolator::has_interpolations(&self.url) ||
    Interpolator::has_interpolations(&self.body.clone().unwrap_or("".to_string())) ||
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
      .buffer_unordered(CONCURRENCY)
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

    // let resp = Response::new().with_status(StatusCode::NotFound);
    // futures::future::ok(resp);

    tokio::run(work);
  }
}
