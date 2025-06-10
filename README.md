# Async Rust Performance Benchmarks

> Benchmarking the performance of async Rust primitives and patterns.

## Benchmarks
| Benchmark | Description | Code | Results |
|-----------|-------------|------|-------|
| [`01.rs`](benches/01.rs) | `Future` implementation vs. `tokio::select!` loop for actors | [`src/_01/`](src/_01) | [`README.md`](src/_01/README.md) |
| [`02.rs`](benches/02.rs) | `JoinSet` vs. `FuturesUnordered` |  |  |
| [`03.rs`](benches/03.rs) | The cost of `Pin<Box<dyn Future<Output = ()>>>` (`async-trait` etc) |  |  |