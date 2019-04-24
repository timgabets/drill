use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

use serde_json::Value;
use yaml_rust::Yaml;
use futures::future::Future;

use crate::iteration::Iteration;
use crate::actions::{Report, Runnable};
use crate::expandable::include;
use crate::{config, writer};

use colored::*;

fn thread_func(benchmark: Arc<Vec<Box<(Runnable + Sync + Send)>>>, config: Arc<config::Config>, thread: i64) -> Vec<Report> {
  let delay = config.rampup / config.threads;
  thread::sleep(std::time::Duration::new((delay * thread) as u64, 0));

  let mut global_reports: Vec<Report> = Vec::new();

  if config.throughput {
    // let mut responses: HashMap<String, Value> = HashMap::new();
    // let mut context: HashMap<String, Yaml> = HashMap::new();
    // let mut reports: Vec<Report> = Vec::new();

    // let all = benchmark.iter().map(move |item| {
    //   item.execute(&mut context, &mut responses, &mut reports, &config)
    // });

    // // let uris = std::iter::repeat(0).take(config.iterations as usize);
    // let all = benchmark.iter().map(|item| {
    //     item.async_execute()
    // });
    // let combined_task = iter_ok::<_, ()>(all).for_each(|f| f);

    // // let nums = stream::iter_ok(uris)
    // //   .map(|_n| {
    // //     // TODO: try to avoid those clones
    // //     //let all = vec![futures::future::ok(()), futures::future::ok(())];

    // //     Box::new(combined_task)
    // //   });

    // // let work = nums
    // //   .buffer_unordered(250)
    // //   .for_each(|_n| {
    // //     Ok(())
    // //   });

    // // tokio::run(work);
    // // tokio::run(combined_task);
    // tokio_scoped::scope(|scope| {
    //   scope.spawn(combined_task);
    // });
  } else {
    for idx in 0..config.iterations {
      // let mut responses: HashMap<String, Value> = HashMap::new();
      let mut context: HashMap<String, Yaml> = HashMap::new();
      // let mut reports: Vec<Report> = Vec::new();

      context.insert("iteration".to_string(), Yaml::String((idx + 1).to_string()));
      context.insert("thread".to_string(), Yaml::String(thread.to_string()));
      context.insert("base".to_string(), Yaml::String(config.base.to_string()));

      let iteration = Iteration {
        number: idx,
        responses: HashMap::new(),
        context: context,
        reports: Vec::new(),
      };

      let work = iteration.future(&benchmark);

      println!("FCS: {}", work);

      // let f1 = benchmark.iter().nth(0).unwrap().execute(&mut context, &mut responses, &mut reports, &config);
      // let (_f2, mut context3, mut responses3, mut reports3) = benchmark.iter().nth(1).unwrap().execute(&mut context2, &mut responses2, &mut reports2, &config);
      // let (_f3, _context4, _responses4, _reports4) = benchmark.iter().nth(2).unwrap().execute(&mut context3, &mut responses3, &mut reports3, &config);

      // tokio_scoped::scope(|scope| {
      //   let (mut new_context, mut new_responses, mut new_reports) = scope.spawn(f1);
      // });

      // let all = benchmark.iter().map(|item| {
      //   item.execute(&mut context, &mut responses, &mut reports, &config)
      // });

      //for item in benchmark.iter() {
      //  let work = item.execute(&mut context, &mut responses, &mut reports, &config);

      //  tokio_scoped::scope(|scope| {
      //    scope.spawn(work);
      //  });
      //}

      // global_reports.push(reports);
    }
  }

  global_reports
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
