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
    future::FutureActor,
    select::{BiasedSelectActor, RandomSelectActor},
};

struct Bencher<'a> {
    /// The runtime that runs the actor.
    rt: &'a Runtime,
    /// The sender that sends tasks to the actor.
    task_sender: mpsc::Sender<Instant>,
    /// The receiver that receives results from the actor.
    result_receiver: mpsc::Receiver<Duration>,
}

impl<'a> Bencher<'a> {
    fn benchmark_throughput<F>(
        mut self,
        actor: F,
        num_tasks: usize,
        iters: usize,
    ) -> ThroughputResult
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.rt.spawn(actor);

        let mut measurements = Vec::with_capacity(iters);

        for _ in 0..iters {
            let sender = self.task_sender.clone();
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

        ThroughputResult { measurements }
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
    };

    let bencher = Bencher {
        rt: &actor_runtime,
        task_sender,
        result_receiver,
    };

    let future_result = bencher.benchmark_throughput(actor, NUM_TASKS, ITERATIONS);
    let future_row = future_result.to_row("FutureActor");

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
    };

    let bencher = Bencher {
        rt: &actor_runtime,
        task_sender,
        result_receiver,
    };

    let random_result = bencher.benchmark_throughput(actor.run(), NUM_TASKS, ITERATIONS);
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
    };

    let bencher = Bencher {
        rt: &actor_runtime,
        task_sender,
        result_receiver,
    };

    let biased_result = bencher.benchmark_throughput(actor.run(), NUM_TASKS, ITERATIONS);
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
}

#[derive(Debug)]
struct ThroughputResult {
    /// Measurements.
    measurements: Vec<ThroughputMeasurement>,
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
        }
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
