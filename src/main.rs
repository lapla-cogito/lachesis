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

fn worker_thread_2() {
    println!("Worker thread 2 starting");
    for i in 0..10 {
        println!("Worker 2: {}", i);

        for j in 0..800000 {
            std::hint::black_box(i * j);
        }

        lachesis::check_preemption();
    }
}

fn main_green_thread() {
    let _id1 = lachesis::spawn(worker_thread_1, 2 * 1024 * 1024);
    let _id2 = lachesis::spawn(worker_thread_2, 2 * 1024 * 1024);

    let thread_name = "Closure Worker 1";
    let iteration_count = 8;
    let work_amount = 900000;

    let _id4 = lachesis::spawn(
        move || {
            println!("{} starting", thread_name);
            for i in 0..iteration_count {
                println!("{}: {}", thread_name, i);

                for j in 0..work_amount {
                    std::hint::black_box(i * j);
                }

                lachesis::check_preemption();
            }
            println!("{} finished", thread_name);
        },
        2 * 1024 * 1024,
    );

    let shared_data = [10, 20, 30, 40, 50];
    let multiplier = 2;

    let _id5 = lachesis::spawn(
        move || {
            println!("Closure Worker 2");
            for (idx, &value) in shared_data.iter().enumerate() {
                let result = value * multiplier;
                println!(
                    "Closure Worker 2: data[{}] = {} * {} = {}",
                    idx, value, multiplier, result
                );

                for j in 0..700000 {
                    std::hint::black_box(result * j);
                }

                lachesis::check_preemption();
            }
            println!("Closure Worker 2 finished");
        },
        2 * 1024 * 1024,
    );

    // Let the main thread do some work
    for i in 0..5 {
        println!("Main Thread: {}", i);

        for j in 0..600000 {
            std::hint::black_box(i * j);
            if j % 60000 == 0 {
                lachesis::check_preemption();
            }
        }
    }
    // Continue with main thread work
    for i in 5..8 {
        println!("Main Thread: {}", i);
        for j in 0..400000 {
            std::hint::black_box(i * j);
        }
    }
}

fn main() {
    println!("\n--- Running Cooperative Scheduler ---");
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

    println!("\n--- Running Green Thread Scheduler ---");

    let scheduler = lachesis::Lachesis::builder()
        .stack_size(4 * 1024 * 1024) // 4MB
        .preemption_interval(10)
        .build();

    if let Err(e) = scheduler.run(main_green_thread) {
        eprintln!("Failed to run green thread scheduler: {}", e);
        if !e.is_recoverable() {
            println!("Error is not recoverable, exiting...");
            return;
        }
    }

    println!("\nAll done!");

    std::process::exit(0);
}
