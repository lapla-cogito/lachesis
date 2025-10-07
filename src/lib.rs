pub mod scheduler;

mod context;
mod cooperative;
mod error;
mod runtime;
mod timer;
mod types;

pub use cooperative::CooperativeScheduler;
pub use runtime::{spawn, spawn_from_main};
pub use scheduler::Lachesis;
pub use timer::{check_preemption, disable_preemption, enable_preemption_with_interval};
pub use types::{SchedulerConfig, Task, ThreadId, ThreadInfo, ThreadState};

#[cfg(test)]
mod tests {
    #[test]
    fn test_cooperative_scheduler() {
        let scheduler = crate::cooperative::CooperativeScheduler::new();
        let counter = std::sync::Arc::new(std::sync::Mutex::new(0));

        for _ in 0..5 {
            let counter_clone = std::sync::Arc::clone(&counter);
            scheduler
                .add_task(std::boxed::Box::new(move || {
                    let mut num = counter_clone.lock().unwrap();
                    *num += 1;
                }))
                .unwrap();
        }

        scheduler.run().unwrap();

        assert_eq!(*counter.lock().unwrap(), 5);
    }

    #[test]
    fn test_green_threads() {
        fn test_thread() {
            for i in 0..3 {
                println!("Green thread: {}", i);
                crate::runtime::schedule();
            }
        }

        crate::runtime::spawn_from_main(test_thread, 2 * 1024 * 1024, 10);
    }
}
