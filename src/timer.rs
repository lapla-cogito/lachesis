static mut TIMER_THREAD_HANDLE: Option<std::thread::JoinHandle<()>> = None;
static mut TIMER_ENABLED: bool = false;

extern "C" fn preemption_signal_handler(_: i32) {
    crate::types::PREEMPTION_REQUESTED.store(true, std::sync::atomic::Ordering::Relaxed);
}

pub fn init_timer(interval_ms: u64) {
    unsafe {
        // signal handler
        let handler = nix::sys::signal::SigHandler::Handler(preemption_signal_handler);
        nix::sys::signal::signal(nix::sys::signal::Signal::SIGALRM, handler).unwrap();

        // reset flags
        crate::types::TIMER_STOP_FLAG.store(false, std::sync::atomic::Ordering::Relaxed);
        crate::types::PREEMPTION_REQUESTED.store(false, std::sync::atomic::Ordering::Relaxed);

        let handle = std::thread::spawn(move || {
            let interval = std::time::Duration::from_millis(interval_ms);

            while !crate::types::TIMER_STOP_FLAG.load(std::sync::atomic::Ordering::Relaxed) {
                std::thread::sleep(interval);

                if !crate::types::TIMER_STOP_FLAG.load(std::sync::atomic::Ordering::Relaxed) {
                    let pid = nix::unistd::getpid();
                    let _ = nix::sys::signal::kill(pid, nix::sys::signal::Signal::SIGALRM);
                }
            }
        });

        let handle_ptr = &raw mut TIMER_THREAD_HANDLE;
        *handle_ptr = Some(handle);

        let timer_ptr = &raw mut TIMER_ENABLED;
        *timer_ptr = true;
    }
}

// indicate safe preemption points
pub fn check_preemption() {
    if !is_preemption_enabled() {
        return;
    }

    // Check atomic flag set by signal handler
    if crate::types::PREEMPTION_REQUESTED.load(std::sync::atomic::Ordering::Acquire) {
        crate::types::PREEMPTION_REQUESTED.store(false, std::sync::atomic::Ordering::Release);
        crate::runtime::schedule();
    }
}

pub fn enable_preemption_with_interval(interval_ms: u64) {
    unsafe {
        let timer_ptr = &raw mut TIMER_ENABLED;
        *timer_ptr = true;
        crate::types::PREEMPTION_REQUESTED.store(false, std::sync::atomic::Ordering::Relaxed);
        init_timer(interval_ms);
    }
}

pub fn disable_preemption() {
    unsafe {
        crate::types::TIMER_STOP_FLAG.store(true, std::sync::atomic::Ordering::Relaxed);

        let handle_ptr = &raw mut TIMER_THREAD_HANDLE;
        if let Some(handle) = (*handle_ptr).take() {
            let _ = handle.join();
        }

        let timer_ptr = &raw mut TIMER_ENABLED;
        *timer_ptr = false;
        crate::types::PREEMPTION_REQUESTED.store(false, std::sync::atomic::Ordering::Release);
    }
}

pub fn is_preemption_enabled() -> bool {
    unsafe {
        let timer_ptr = &raw const TIMER_ENABLED;
        *timer_ptr
    }
}
