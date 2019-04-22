use futures::Future;
use futures::stream::iter_ok;
use futures::stream::Stream;
use std::sync::Arc;
use crate::actions::Runnable;
use crate::config;

pub fn build(benchmark: Arc<Vec<Box<(Runnable + Sync + Send)>>>, config: Arc<config::Config>) -> Box<Future<Item=(), Error=()> + Send> {
    // let f0 = futures::future::ok(());

    // let all = benchmark.iter().map(|item| item.clone().async_execute());
    let all = vec![futures::future::ok(()), futures::future::ok(())];

    let combined_task = iter_ok::<_, ()>(all).for_each(|f| f);

    Box::new(combined_task)

    //f0
}
