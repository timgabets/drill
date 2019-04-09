use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

use futures::{stream, Future, Stream};
use serde_json::Value;
use yaml_rust::Yaml;

use crate::actions::{Report, Runnable};
use crate::config;
use crate::expandable::include;
use crate::writer;

use colored::*;

fn thread_func(benchmark: Arc<Vec<Box<(Runnable + Sync + Send)>>>, config: Arc<config::Config>, thread: i64) -> Vec<Report> {
  let delay = config.rampup / config.threads;
  thread::sleep(std::time::Duration::new((delay * thread) as u64, 0));

  let mut global_reports = Vec::new();

  if config.throughput {
    let client = hyper::Client::new();
    let uris = std::iter::repeat(0).take(config.iterations as usize);
    let nums = stream::iter_ok(uris)
      .map(move |_n| {
        let f1 = client
          .get("http://localhost:9000/api/users.json".parse().unwrap())
          .and_then(|resp| {
            println!("Status: {}", resp.status());
            futures::future::ok(())
          });

        let f2 = client
          .get("http://localhost:9000/api/organizations".parse().unwrap())
          .and_then(|_resp| {
            f1
          })
          .map_err(|err| {
            println!("Error: {}", err);
          });

        f2
      });

    let work = nums
      .buffer_unordered(250)
      .for_each(|_n| {
        Ok(())
      });

    tokio::run(work);
  } else {
    for iteration in 1..config.iterations {
      let mut responses: HashMap<String, Value> = HashMap::new();
      let mut context: HashMap<String, Yaml> = HashMap::new();
      let mut reports: Vec<Report> = Vec::new();

      context.insert("iteration".to_string(), Yaml::String(iteration.to_string()));
      context.insert("thread".to_string(), Yaml::String(thread.to_string()));
      context.insert("base".to_string(), Yaml::String(config.base.to_string()));

      for item in benchmark.iter() {
        item.execute(&mut context, &mut responses, &mut reports, &config);
      }

      global_reports.push(reports);
    }
  }

  global_reports.concat()
}

fn join<S: ToString>(l: Vec<S>, sep: &str) -> String {
  l.iter().fold("".to_string(),
                  |a,b| if !a.is_empty() {a+sep} else {a} + &b.to_string()
                  )
}

pub fn execute(benchmark_path: &str, report_path_option: Option<&str>, no_check_certificate: bool, quiet: bool, nanosec: bool, throughput: bool) -> Result<Vec<Vec<Report>>, Vec<Vec<Report>>> {
  let config = Arc::new(config::Config::new(benchmark_path, no_check_certificate, quiet, nanosec, throughput));

  if report_path_option.is_some() {
    println!("{}: {}. Ignoring {} and {} properties...", "Report mode".yellow(), "on".purple(), "threads".yellow(), "iterations".yellow());
  } else {
    println!("{} {}", "Threads".yellow(), config.threads.to_string().purple());
    println!("{} {}", "Iterations".yellow(), config.iterations.to_string().purple());
    println!("{} {}", "Rampup".yellow(), config.rampup.to_string().purple());
  }

  println!("{} {}", "Base URL".yellow(), config.base.purple());
  println!("");

  let mut list: Vec<Box<(Runnable + Sync + Send)>> = Vec::new();

  include::expand_from_filepath(benchmark_path, &mut list, Some("plan"));

  if config.throughput && list.iter().any(|item| item.has_interpolations()) {
    panic!("Throughput mode incompatible with interpolations!");
  }

  let list_arc = Arc::new(list);
  let mut children = vec![];
  let mut list_reports: Vec<Vec<Report>> = vec![];

  if let Some(report_path) = report_path_option {
    let reports = thread_func(list_arc.clone(), config, 0);

    writer::write_file(report_path, join(reports, ""));

    Ok(list_reports)
  } else {
    for index in 0..config.threads {
      let list_clone = list_arc.clone();
      let config_clone = config.clone();
      children.push(thread::spawn(move || thread_func(list_clone, config_clone, index)));
    }

    for child in children {
      // Wait for the thread to finish. Returns a result.
      let thread_result = child.join();

      match thread_result {
        Ok(v) => list_reports.push(v),
        Err(_) => panic!("arrrgh"),
      }
    }

    Ok(list_reports)
  }
}
