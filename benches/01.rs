use futures::stream::FuturesUnordered;
use std::time::{Duration, Instant};
use tabled::{
    Table, Tabled,
    settings::{Color, Style, object::Rows},
};
use tokio::{
    runtime::{Builder, Runtime},
    sync::mpsc,
};

use async_rust_benchmarks::_01::{
    Actor, ActorMetrics,
    future::FutureActor,
    select::{BiasedSelectActor, RandomSelectActor},
};

struct Bencher<'a> {
    /// The runtime that runs the actor.
    rt: &'a Runtime,
    /// The sender that sends tasks to the actor.
    task_sender: Option<mpsc::Sender<Instant>>,
    /// The receiver that receives results from the actor.
    result_receiver: mpsc::Receiver<Duration>,
}

impl<'a> Bencher<'a> {
    fn benchmark_throughput<A: Actor>(
        &mut self,
        actor: A,
        num_tasks: usize,
        iters: usize,
    ) -> ThroughputResult {
        let handle = self.rt.spawn(actor.run());

        // Take the sender
        let task_sender = self.task_sender.take().unwrap();

        let mut measurements = Vec::with_capacity(iters);

        for _ in 0..iters {
            let sender = task_sender.clone();
            let start = self.rt.spawn(async move {
                // Start here, don't want to measure the `spawn` duration.
                let start = Instant::now();

                for _ in 0..num_tasks {
                    sender.send(Instant::now()).await.unwrap();
                }

                start
            });

            for _ in 0..num_tasks {
                let _ = self.rt.block_on(self.result_receiver.recv());
            }

            let end = Instant::now();
            let start = self.rt.block_on(start).unwrap();

            let elapsed = end.duration_since(start);
            let throughput = num_tasks as f64 / elapsed.as_secs_f64();

            measurements.push(ThroughputMeasurement {
                elapsed,
                throughput,
            });
        }

        drop(task_sender);
        let metrics = self.rt.block_on(handle).unwrap();

        ThroughputResult {
            measurements,
            metrics,
        }
    }

    /// This benchmark measures the individual latency of each task, where latency is defined as the time between
    /// - The task being sent to the actor
    /// - The task being processed by the actor and queued on the result channel (NOT the last mile result communication)
    fn benchmark_latency<A: Actor>(
        &mut self,
        actor: A,
        num_tasks: usize,
        iters: usize,
    ) -> LatencyResult {
        self.rt.spawn(actor.run());
        let task_sender = self.task_sender.take().unwrap();

        let mut measurements = Vec::with_capacity(iters * num_tasks);

        for _ in 0..iters {
            let sender = task_sender.clone();
            self.rt.spawn(async move {
                for _ in 0..num_tasks {
                    sender.send(Instant::now()).await.unwrap();
                }
            });

            for _ in 0..num_tasks {
                let result = self.rt.block_on(self.result_receiver.recv()).unwrap();
                measurements.push(result);
            }
        }

        LatencyResult { measurements }
    }
}

fn main() {
    const NUM_TASKS: usize = 50000;
    const ITERATIONS: usize = 100;

    let actor_runtime = Builder::new_current_thread().enable_time().build().unwrap();

    let (task_sender, task_receiver) = mpsc::channel(NUM_TASKS);
    let (result_sender, result_receiver) = mpsc::channel(NUM_TASKS);

    let actor = FutureActor {
        incoming_tasks: task_receiver,
        processing_tasks: FuturesUnordered::new(),
        results: result_sender,
        metrics: ActorMetrics::new(),
    };

    let mut bencher = Bencher {
        rt: &actor_runtime,
        task_sender: Some(task_sender),
        result_receiver,
    };

    let throughput_result = bencher.benchmark_throughput(actor, NUM_TASKS, ITERATIONS);
    let future_row = throughput_result.to_row("FutureActor");

    println!(
        "{}",
        Table::new(vec![future_row.clone()]).with(Style::modern())
    );

    let (task_sender, task_receiver) = mpsc::channel(NUM_TASKS);
    let (result_sender, result_receiver) = mpsc::channel(NUM_TASKS);

    let actor = RandomSelectActor {
        incoming_tasks: task_receiver,
        processing_tasks: FuturesUnordered::new(),
        results: result_sender,
        metrics: ActorMetrics::new(),
    };

    let mut bencher = Bencher {
        rt: &actor_runtime,
        task_sender: Some(task_sender),
        result_receiver,
    };

    let random_result = bencher.benchmark_throughput(actor, NUM_TASKS, ITERATIONS);
    let random_row = random_result.to_row("RandomSelectActor");

    println!(
        "{}",
        Table::new(vec![random_row.clone()]).with(Style::modern())
    );

    let (task_sender, task_receiver) = mpsc::channel(NUM_TASKS);
    let (result_sender, result_receiver) = mpsc::channel(NUM_TASKS);

    let actor = BiasedSelectActor {
        incoming_tasks: task_receiver,
        processing_tasks: FuturesUnordered::new(),
        results: result_sender,
        metrics: ActorMetrics::new(),
    };

    let mut bencher = Bencher {
        rt: &actor_runtime,
        task_sender: Some(task_sender),
        result_receiver,
    };

    let biased_result = bencher.benchmark_throughput(actor, NUM_TASKS, ITERATIONS);
    let biased_row = biased_result.to_row("BiasedSelectActor");

    println!(
        "{}",
        Table::new(vec![biased_row.clone()]).with(Style::modern())
    );

    let mut rows = vec![future_row, random_row, biased_row];

    rows.sort_by_key(|row| row.mean_duration);
    let mut table = Table::new(rows);
    table.modify(Rows::one(1), Color::BOLD);

    println!("{}", table.with(Style::modern()));

    let (task_sender, task_receiver) = mpsc::channel(NUM_TASKS);
    let (result_sender, result_receiver) = mpsc::channel(NUM_TASKS);

    let actor = FutureActor {
        incoming_tasks: task_receiver,
        processing_tasks: FuturesUnordered::new(),
        results: result_sender,
        metrics: ActorMetrics::new(),
    };

    let mut bencher = Bencher {
        rt: &actor_runtime,
        task_sender: Some(task_sender),
        result_receiver,
    };

    let latency_result = bencher.benchmark_latency(actor, NUM_TASKS, ITERATIONS);
    let future_latency_row = latency_result.to_row("FutureActor");

    println!(
        "{}",
        Table::new(vec![future_latency_row.clone()]).with(Style::modern())
    );

    let (task_sender, task_receiver) = mpsc::channel(NUM_TASKS);
    let (result_sender, result_receiver) = mpsc::channel(NUM_TASKS);

    let actor = RandomSelectActor {
        incoming_tasks: task_receiver,
        processing_tasks: FuturesUnordered::new(),
        results: result_sender,
        metrics: ActorMetrics::new(),
    };

    let mut bencher = Bencher {
        rt: &actor_runtime,
        task_sender: Some(task_sender),
        result_receiver,
    };

    let latency_result = bencher.benchmark_latency(actor, NUM_TASKS, ITERATIONS);
    let random_latency_row = latency_result.to_row("RandomSelectActor");

    println!(
        "{}",
        Table::new(vec![random_latency_row.clone()]).with(Style::modern())
    );

    let (task_sender, task_receiver) = mpsc::channel(NUM_TASKS);
    let (result_sender, result_receiver) = mpsc::channel(NUM_TASKS);

    let actor = BiasedSelectActor {
        incoming_tasks: task_receiver,
        processing_tasks: FuturesUnordered::new(),
        results: result_sender,
        metrics: ActorMetrics::new(),
    };

    let mut bencher = Bencher {
        rt: &actor_runtime,
        task_sender: Some(task_sender),
        result_receiver,
    };

    let latency_result = bencher.benchmark_latency(actor, NUM_TASKS, ITERATIONS);
    let biased_latency_row = latency_result.to_row("BiasedSelectActor");

    println!(
        "{}",
        Table::new(vec![biased_latency_row.clone()]).with(Style::modern())
    );

    let mut rows = vec![future_latency_row, random_latency_row, biased_latency_row];

    rows.sort_by_key(|row| row.median_latency);
    let mut table = Table::new(rows);
    table.modify(Rows::one(1), Color::BOLD);

    println!("{}", table.with(Style::modern()));
}

#[derive(Debug)]
struct ThroughputResult {
    /// Measurements.
    measurements: Vec<ThroughputMeasurement>,
    /// Metrics.
    metrics: ActorMetrics,
}

impl ThroughputResult {
    fn to_row(&self, actor_type: &'static str) -> ThroughputRow {
        ThroughputRow {
            actor_type,
            mean_duration: self.mean_duration(),
            mean_throughput: self.mean_throughput(),
            median_duration: self.median_duration(),
            median_throughput: self.median_throughput(),
            min_duration: self.min_duration(),
            max_duration: self.max_duration(),
            min_throughput: self.min_throughput(),
            max_throughput: self.max_throughput(),
            max_pending_tasks: self.metrics.max_pending_tasks(),
        }
    }
}

#[derive(Debug)]
struct LatencyResult {
    /// Measurements.
    measurements: Vec<Duration>,
}

impl LatencyResult {
    fn to_row(&self, actor_type: &'static str) -> LatencyRow {
        LatencyRow {
            actor_type,
            mean_latency: self.mean_latency(),
            median_latency: self.quantile(0.5),
            min_latency: self.min_latency(),
            p10_latency: self.quantile(0.1),
            p90_latency: self.quantile(0.9),
            p99_latency: self.quantile(0.99),
            max_latency: self.max_latency(),
        }
    }
}

impl LatencyResult {
    fn mean_latency(&self) -> Duration {
        self.measurements.iter().sum::<Duration>() / self.measurements.len() as u32
    }

    fn quantile(&self, quantile: f64) -> Duration {
        let mut durations = self.measurements.clone();
        durations.sort();
        let index = ((durations.len() - 1) as f64 * quantile).round() as usize;
        durations[index]
    }

    fn min_latency(&self) -> Duration {
        *self.measurements.iter().min().unwrap()
    }

    fn max_latency(&self) -> Duration {
        *self.measurements.iter().max().unwrap()
    }
}

fn format_duration(duration: &Duration) -> String {
    format!("{:.2?}", duration)
}

fn format_throughput(throughput: &f64) -> String {
    if *throughput > 1_000_000.0 {
        format!("{:.3}M", throughput / 1_000_000.0)
    } else if *throughput > 1_000.0 {
        format!("{:.3}k", throughput / 1000.0)
    } else {
        format!("{:.3}", throughput)
    }
}

#[derive(Debug, Tabled, Clone)]
struct ThroughputRow {
    /// Name of the actor.
    actor_type: &'static str,
    /// Mean duration.
    #[tabled(display = "format_duration")]
    mean_duration: Duration,
    /// Mean throughput.
    #[tabled(display = "format_throughput")]
    mean_throughput: f64,
    /// Median duration.
    #[tabled(display = "format_duration")]
    median_duration: Duration,
    /// Median throughput.
    #[tabled(display = "format_throughput")]
    median_throughput: f64,
    /// Min duration.
    #[tabled(display = "format_duration")]
    min_duration: Duration,
    /// Max duration.
    #[tabled(display = "format_duration")]
    max_duration: Duration,
    /// Min throughput.
    #[tabled(display = "format_throughput")]
    min_throughput: f64,
    /// Max throughput.
    #[tabled(display = "format_throughput")]
    max_throughput: f64,
    /// Max pending tasks.
    max_pending_tasks: usize,
}

#[derive(Debug, Tabled, Clone)]
struct LatencyRow {
    /// Name of the actor.
    actor_type: &'static str,
    /// Mean latency.
    #[tabled(display = "format_duration")]
    mean_latency: Duration,
    /// Median latency.
    #[tabled(display = "format_duration")]
    median_latency: Duration,
    /// Min latency.
    #[tabled(display = "format_duration")]
    min_latency: Duration,
    /// Max latency.
    #[tabled(display = "format_duration")]
    max_latency: Duration,
    /// 10th percentile latency.
    #[tabled(display = "format_duration")]
    p10_latency: Duration,
    /// 90th percentile latency.
    #[tabled(display = "format_duration")]
    p90_latency: Duration,
    /// 99th percentile latency.
    #[tabled(display = "format_duration")]
    p99_latency: Duration,
}

#[derive(Debug)]
struct ThroughputMeasurement {
    /// Total elapsed time.
    elapsed: Duration,
    /// Throughput in completed tasks per second.
    throughput: f64,
}

impl ThroughputResult {
    fn mean_duration(&self) -> Duration {
        self.measurements
            .iter()
            .map(|m| m.elapsed)
            .sum::<Duration>()
            / self.measurements.len() as u32
    }

    fn mean_throughput(&self) -> f64 {
        self.measurements.iter().map(|m| m.throughput).sum::<f64>() / self.measurements.len() as f64
    }

    fn median_duration(&self) -> Duration {
        let mut durations = self
            .measurements
            .iter()
            .map(|m| m.elapsed)
            .collect::<Vec<_>>();
        durations.sort();
        durations[durations.len() / 2]
    }

    fn median_throughput(&self) -> f64 {
        let mut throughputs = self
            .measurements
            .iter()
            .map(|m| m.throughput)
            .collect::<Vec<_>>();
        throughputs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        throughputs[throughputs.len() / 2]
    }

    fn min_duration(&self) -> Duration {
        self.measurements.iter().map(|m| m.elapsed).min().unwrap()
    }

    fn max_duration(&self) -> Duration {
        self.measurements.iter().map(|m| m.elapsed).max().unwrap()
    }

    fn min_throughput(&self) -> f64 {
        self.measurements
            .iter()
            .map(|m| m.throughput)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    }

    fn max_throughput(&self) -> f64 {
        self.measurements
            .iter()
            .map(|m| m.throughput)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    }
}
