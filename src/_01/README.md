# 01: `Future` implementation vs. `tokio::select!` loop for actors

- [Overview](#overview)
- [Results](#results)

## Overview
This benchmark compares the performance of a `Future` implementation vs. a `tokio::select!` loop for long-running actors.
The basic flow looks like this:

```mermaid
graph TD
    A["Client"] --> B["Task Channel<br/>(mpsc::Receiver)"]
    B --> C["Actor<br/>(Future vs tokio::select!)"]
    C --> D["Processing Tasks<br/>(FuturesUnordered)"]
    C --> E["Results Channel<br/>(mpsc::Sender)"]
    E --> F["Client Results<br/>(mpsc::Receiver)"]
    
    style A fill:#e1f5fe
    style F fill:#e8f5e8
    style C fill:#fff3e0
    style D fill:#fce4ec
```

The workload is a simple task that multiplies a number by 2 and adds 10 microseconds of delay (in the form of `tokio::time::sleep`).

## Results
### Latency
TODO
### Throughput
TODO
### Memory Usage
TODO

## Notes
- Understand why `tokio::unconstrained` on the `FutureActor` is necessary to get the same performance as the `tokio::select!` loop.