pub struct CooperativeScheduler {
    queue: std::sync::Arc<std::sync::Mutex<std::collections::VecDeque<crate::types::Task>>>,
}

impl Default for CooperativeScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl CooperativeScheduler {
    pub fn new() -> Self {
        CooperativeScheduler {
            queue: std::sync::Arc::new(std::sync::Mutex::new(std::collections::VecDeque::new())),
        }
    }

    pub fn add_task(&self, task: crate::types::Task) -> crate::error::Result<()> {
        let mut q = self
            .queue
            .lock()
            .map_err(|_| crate::error::Error::LockFailed)?;
        q.push_back(task);

        Ok(())
    }

    pub fn run(&self) -> crate::error::Result<()> {
        while let Some(task) = {
            let mut q = self
                .queue
                .lock()
                .map_err(|_| crate::error::Error::LockFailed)?;
            q.pop_front()
        } {
            task();
        }

        Ok(())
    }
}
