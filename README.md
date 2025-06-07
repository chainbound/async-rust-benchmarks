# Async Rust Performance Benchmarks

- Benchmarking the performance of async Rust primitives.
- Use samply
- Use criterion for both performance and memory usage.

## Benchmarks
| Benchmark | Description |
|-----------|-------------|
| [01.rs](benches/01.rs) | `Future` implementation vs. `tokio::select!` loop |
| [02.rs](benches/02.rs) | `JoinSet` vs. `FuturesUnordered` |
| [03.rs](benches/03.rs) |  |

## To Do
- [ ] Persist benchmark results to a file.
- [ ] Run benchmarks in CI.