# Async Rust Performance Benchmarks

- Benchmarking the performance of async Rust primitives.
- Use samply
- Use criterion for both performance and memory usage.

## Benchmarks
| Benchmark | Description | Code |
|-----------|-------------|------|
| [`01.rs`](benches/01.rs) | `Future` implementation vs. `tokio::select!` loop for actors | [`src/_01/`](src/_01) |
| [`02.rs`](benches/02.rs) | `JoinSet` vs. `FuturesUnordered` |  |

## To Do
- [ ] Persist benchmark results to a file.
- [ ] Run benchmarks in CI.
- [ ] Benchmark memory usage.