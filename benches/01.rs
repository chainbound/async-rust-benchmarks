use futures::stream::FuturesUnordered;
use std::hint::black_box;
use tokio::{runtime::Runtime, sync::mpsc};

use criterion::{Criterion, criterion_group, criterion_main};

use async_rust_benchmarks::_01::{future::FutureActor, select::RandomSelectActor};

async fn run_future_actor(num_tasks: u64) {
    let (task_sender, task_receiver) = mpsc::channel(num_tasks as usize);
    let (result_sender, mut result_receiver) = mpsc::channel(num_tasks as usize);

    let actor = FutureActor {
        incoming_tasks: task_receiver,
        processing_tasks: FuturesUnordered::new(),
        results: result_sender,
    };

    // Run the actor in the background
    tokio::spawn(actor);

    // Send tasks
    for i in 1..=num_tasks {
        task_sender.send(i).await.unwrap();
    }

    // Collect results
    let mut results = Vec::new();
    while results.len() < num_tasks as usize {
        if let Some(result) = result_receiver.recv().await {
            results.push(result);
        } else {
            break;
        }
    }
}

async fn run_random_select_actor(num_tasks: u64) {
    let (task_sender, task_receiver) = mpsc::channel(num_tasks as usize);
    let (result_sender, mut result_receiver) = mpsc::channel(num_tasks as usize);

    let actor = RandomSelectActor {
        incoming_tasks: task_receiver,
        processing_tasks: FuturesUnordered::new(),
        results: result_sender,
    };

    tokio::spawn(actor.run());

    // Send tasks
    for i in 1..=num_tasks {
        task_sender.send(i).await.unwrap();
    }

    // Collect results
    let mut results = Vec::new();
    while results.len() < num_tasks as usize {
        if let Some(result) = result_receiver.recv().await {
            results.push(result);
        } else {
            break;
        }
    }
}

fn bench_future_actor(c: &mut Criterion) {
    // Create the Tokio runtime
    let rt = Runtime::new().unwrap();

    c.bench_function("future_actor_10000_tasks", |b| {
        b.to_async(&rt).iter(|| run_future_actor(black_box(10000)))
    });

    c.bench_function("random_select_actor_10000_tasks", |b| {
        b.to_async(&rt)
            .iter(|| run_random_select_actor(black_box(10000)))
    });
}

criterion_group!(benches, bench_future_actor);
criterion_main!(benches);
