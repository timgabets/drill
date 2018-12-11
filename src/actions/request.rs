use std::collections::HashMap;
use std::io::{self, Write};

use yaml_rust::Yaml;
use colored::*;
use serde_json;
// use time;

use hyper::{Client, Method, Body};
use hyper::header::HeaderName;
use hyper::rt::{Future, Stream};

use interpolator;
use config;

use actions::{Runnable, Report};

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
  pub fn is_that_you(item: &Yaml) -> bool{
    item["request"].as_hash().is_some()
  }

  pub fn new(item: &Yaml, with_item: Option<Yaml>) -> Request {
    let reference: Option<&str> = item["assign"].as_str();
    let body: Option<&str> = item["request"]["body"].as_str();
    let method;

    let mut headers = HashMap::new();

    if let Some(v) = item["request"]["method"].as_str() {
      method = v.to_string().to_uppercase();
    } else {
      method = "GET".to_string();
    }

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

  fn send_request(&self, context: &mut HashMap<String, Yaml>, responses: &mut HashMap<String, serde_json::Value>, _config: &config::Config) {
    let client = Client::new();

    let begin = time::precise_time_s();

    let interpolated_name;
    let interpolated_url;
    // let interpolated_body;
    // let request;

    // Resolve the url
    {
      let interpolator = interpolator::Interpolator::new(context, responses);
      interpolated_name = interpolator.resolve(&self.name);
      interpolated_url = interpolator.resolve(&self.url);
    }

    // Method
    let method = match self.method.to_uppercase().as_ref() {
      "GET" => Method::GET,
      "POST" => Method::POST,
      "PUT" => Method::PUT,
      "PATCH" => Method::PATCH,
      "DELETE" => Method::DELETE,
      _ => panic!("Unknown method '{}'", self.method),
    };

    let mut builder = hyper::Request::builder();

    builder.uri(interpolated_url.clone());
    builder.method(method);

    // let req = builder.body(Body::from(""));

    // Body
    // if let Some(body) = self.body.as_ref() {
    //   // Resolve the body
    //   let interpolator = interpolator::Interpolator::new(context, responses);
    //   interpolated_body = interpolator.resolve(body).to_owned();

    //   request = client
    //     .method(method)
    //     .uri(&interpolated_url)
    //     .body(&interpolated_body);
    // } else {
    //   request = client.request(method, &interpolated_url);
    // }

    // Headers
    builder.header(hyper::header::USER_AGENT, USER_AGENT.to_string());

    if let Some(cookie) = context.get("cookie") {
      builder.header(
        hyper::header::COOKIE,
        cookie.as_str().unwrap()
      );
    }

    for (key, val) in self.headers.iter() {
      // Resolve the body
      let interpolator = interpolator::Interpolator::new(context, responses);
      let interpolated_header = interpolator.resolve(val);

      builder.header(
        HeaderName::from_lowercase(key.as_bytes()).unwrap(),
        interpolated_header
      );
    }

    let req = builder.body(Body::from(""));

    let future = client
      .request(req.unwrap())
      .and_then(move |res| {
          let duration_ms = (time::precise_time_s() - begin) * 1000.0;
          let status = res.status();

          let status_text = if status.is_server_error() {
            status.to_string().red()
          } else if status.is_client_error() {
            status.to_string().purple()
          } else {
            status.to_string().yellow()
          };

          println!("{:width$} {} {} {}{}", interpolated_name.green(), interpolated_url.blue().bold(), status_text, duration_ms.round().to_string().cyan(), "ms".cyan(), width=25);

          // The body is a stream, and for_each returns a new Future
          // when the stream is finished, and calls the closure on
          // each chunk of the body...
          res.into_body().for_each(|chunk| {
              io::stdout().write_all(&chunk)
                  .map_err(|e| panic!("example expects stdout is open, error={}", e))
          })
      })
      .map_err(|err| {
          eprintln!("Error {}", err);
      });

    // let future = client
    //   .request(request.unwrap())
    //   .and_then(|res| {
    //     println!("Response: {}", res.status());
    //   });

      // .map(|_| {
      //   println!("\n\nDone.");
      // })
      // .map_err(|err| {
      //   eprintln!("Error {}", err);
      // });

    hyper::rt::run(future);

    // if let Err(e) = response_result {
    //   panic!("Error connecting '{}': {:?}", interpolated_url, e);
    // }

    // let response = response_result.unwrap();


    // (response, duration_ms)
  }
}

impl Runnable for Request {
  fn execute(&self, context: &mut HashMap<String, Yaml>, responses: &mut HashMap<String, serde_json::Value>, _reports: &mut Vec<Report>, config: &config::Config) {
    if self.with_item.is_some() {
      context.insert("item".to_string(), self.with_item.clone().unwrap());
    }

    self.send_request(context, responses, config);

    // reports.push(Report { name: self.name.to_owned(), duration: duration_ms, status: response.status.to_u16() });

    // if let Some(&SetCookie(ref cookies)) = response.headers.get::<SetCookie>() {
    //   if let Some(cookie) = cookies.iter().next() {
    //     let value = String::from(cookie.split(";").next().unwrap());
    //     context.insert("cookie".to_string(), Yaml::String(value));
    //   }
    // }

    // if let Some(ref key) = self.assign {
    //   let mut data = String::new();

    //   response.read_to_string(&mut data).unwrap();

    //   let value: serde_json::Value = serde_json::from_str(&data).unwrap();

    //   responses.insert(key.to_owned(), value);
    // }
  }

}
