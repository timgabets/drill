use futures::Future;
use std::sync::Arc;
use crate::actions::Runnable;
use crate::config;

pub fn build(benchmark: Arc<Vec<Box<(Runnable + Sync + Send)>>>, config: Arc<config::Config>) -> impl Future<Item=(), Error=()> {

    // let client = hyper::Client::new();
    // client
    //   .get("http://localhost:9000/api/organizations".parse().unwrap())
    //   .map_err(|err| {
    //     println!("Error: {}", err);
    //   })

    // benchmark
    //   .iter()
    //   .for_each(|def| {
    //     let client = hyper::Client::new();

    //     let req = client
    //       .get("http://localhost:9000/api/organizations".parse().unwrap())
    //       .and_then(|resp| {
    //         println!("Status: {}", resp.status());

    //         previous.unwrap_or(last_ok)
    //       });

    //     previous = Some(req);
    //   });

    // let all = benchmark
    //   .iter()
    //   .map(|def| {
    //     let client = hyper::Client::new();

    //     client
    //       .get("http://localhost:9000/api/organizations".parse().unwrap())
    //       .map_err(|err| {
    //         println!("Error: {}", err);
    //       })
    //   });

    // futures::future::join_all(all).then(|a| { () })

    let client = hyper::Client::new();
    let f0 = futures::future::ok(());

    let _all = benchmark.iter().map(|item| item.async_execute() );

    let f1 = client
      .get("http://localhost:9000/api/users.json".parse().unwrap())
      .and_then(|resp| {
        println!("Status: {}", resp.status());

        f0
      });

    let f2 = client
      .get("http://localhost:9000/api/organizations".parse().unwrap())
      .and_then(|resp| {
        println!("Status: {}", resp.status());

        f1
      });

    let f3 = client
      .get("http://localhost:9000/api/comments.json".parse().unwrap())
      .and_then(|resp| {
        println!("Status: {}", resp.status());

        f2
      });

    f3.map_err(|err| {
      println!("Error: {}", err);
    })
}
