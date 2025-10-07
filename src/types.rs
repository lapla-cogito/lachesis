pub type Task = Box<dyn FnOnce() + Send + 'static>;
pub type ThreadId = u64;
pub type Entry = fn();

pub trait Executable: Send + 'static {
    fn execute(self: Box<Self>);
}

impl<F> Executable for F
where
    F: FnOnce() + Send + 'static,
{
    fn execute(self: Box<Self>) {
        self();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadState {
    Ready,
    Running,
    Terminated,
}

#[derive(Debug, Clone)]
pub struct ThreadInfo {
    pub id: ThreadId,
    pub state: ThreadState,
}

#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    pub default_stack_size: usize,
    pub preemption_interval_ms: u64,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        SchedulerConfig {
            default_stack_size: 2 * 1024 * 1024, // 2MB
            preemption_interval_ms: 10,
        }
    }
}

pub static TIMER_STOP_FLAG: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
pub static PREEMPTION_REQUESTED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
