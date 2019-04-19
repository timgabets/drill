use futures::{Future, Stream};
use std::sync::Arc;
use crate::actions::Runnable;
use crate::config;

pub fn build(benchmark: Arc<Vec<Box<(Runnable + Sync + Send)>>>, config: Arc<config::Config>) -> impl Future<Item=(), Error=()> {
    let client = hyper::Client::new();
    let f1 = client
      .get("http://localhost:9000/api/users.json".parse().unwrap())
      .and_then(|resp| {
        println!("Status: {}", resp.status());
        futures::future::ok(())
      });

    let f2 = client
      .get("http://localhost:9000/api/organizations".parse().unwrap())
      .and_then(|resp| {
        println!("Status: {}", resp.status());

        f1
      });

    let f3 = client
      .get("http://localhost:9000/api/comments.json".parse().unwrap())
      .and_then(|_resp| {
        f2
      })
      .map_err(|err| {
        println!("Error: {}", err);
      });

    f3
}
