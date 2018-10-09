# ucore-process
Like memory management, this crate module contains architecture independent
parts of RustOS process management.

One term we use is "Processor". It contains a series of processes running on a
single CPU core, and some metadata for managing them to multiplex the CPU core,
like `current_pid` and a scheduler.

| File           | Description                                                                                                |
| ---            | ---                                                                                                        |
| `event_hub.rs` | A queue containing timers.                                                                                 |
| `processor.rs` | Defines the structs `Process`, `Context` (wraps arch-dependent details and an address space), `Processor`, |
| `scheduler.rs` | The scheduler framework and two implementations, round-robin and stride.                                   |
| `thread.rs`    | From basic support features builds a `Thread` that has standard interfaces identical to `std::thread`      |
