# lachesis

A toy thread scheduler written in Rust that supports both cooperative and preemptive scheduling.

## Cooperative scheduling example

```rust
let coop_scheduler = lachesis::CooperativeScheduler::new();

for i in 0..3 {
    if let Err(e) = coop_scheduler.add_task(Box::new(move || {
        for j in 0..3 {
            std::hint::black_box(i * j);
            println!("Task {}: Executing inner loop {}", i, j);
        }
    })) {
        eprintln!("Failed to add task: {}", e);
        if !e.is_recoverable() {
            return;
        }
    }
}

if let Err(e) = coop_scheduler.run() {
    eprintln!("Failed to run cooperative scheduler: {}", e);
}
```

## Preemptive scheduling example

This scheduler can use both function pointers and closures as worker threads. Currently, each worker explicitly declares points where the scheduler may intervene in a checkpoint format. In this sense, this scheduler is partially cooperative.

```rust
let scheduler = lachesis::Lachesis::builder()
    .stack_size(4 * 1024 * 1024) // 4MB
    .preemption_interval(10) // 10ms
    .build();

if let Err(e) = scheduler.run(main_green_thread) {
    eprintln!("Failed to run green thread scheduler: {}", e);
    if !e.is_recoverable() {
        println!("Error is not recoverable, exiting...");
        return;
    }
}

fn main_green_thread() {
    // Spawn a worker thread using a function pointer
    let _id1 = lachesis::spawn(worker_thread_1, 2 * 1024 * 1024);

    let shared_data = vec![1, 2, 3, 4, 5];
    let multiplier = 3;
    // Spawn a worker thread using a closure
    let _id2 = lachesis::spawn(
        move || {
            println!("Closure Worker 2");
            for (idx, &value) in shared_data.iter().enumerate() {
                let result = value * multiplier;
                println!(
                    "Closure Worker 2: Index {}, Value {}, Result {}",
                    idx, value, result
                );

                // Regular safe point after each iteration
                lachesis::check_preemption();
            }
        },
        2 * 1024 * 1024,
    );
}

fn worker_thread_1() {
    println!("Worker thread 1 starting");
    for i in 0..10 {
        println!("Worker 1: {}", i);

        for j in 0..1000000 {
            std::hint::black_box(i * j);
        }

        // Regular safe point after each iteration
        lachesis::check_preemption();
    }
}

```

## Test

```sh
$ cargo test
```

## Run example

```sh
$ cargo run
```

## License

MIT
