pub struct Lachesis {
    config: crate::types::SchedulerConfig,
    initialized: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl Lachesis {
    pub fn new(config: crate::types::SchedulerConfig) -> Self {
        Lachesis {
            config,
            initialized: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }

    pub fn run<F>(&self, main_func: F) -> crate::error::Result<()>
    where
        F: FnOnce() + Send + 'static,
    {
        if self
            .initialized
            .swap(true, std::sync::atomic::Ordering::SeqCst)
        {
            return std::result::Result::Err(crate::error::Error::AlreadyInitialized);
        }

        let stack_size = self.config.default_stack_size;
        if stack_size < 64 * 1024 {
            return std::result::Result::Err(crate::error::Error::InvalidStackSize {
                size: stack_size,
                min: 64 * 1024, // Minimum 64KB
            });
        }

        let preemption_interval = self.config.preemption_interval_ms;

        crate::runtime::execute_main(main_func, stack_size, preemption_interval);

        self.initialized
            .store(false, std::sync::atomic::Ordering::SeqCst);

        Ok(())
    }
}

pub struct ConfigBuilder {
    config: crate::types::SchedulerConfig,
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigBuilder {
    pub fn new() -> Self {
        ConfigBuilder {
            config: crate::types::SchedulerConfig::default(),
        }
    }

    pub fn stack_size(mut self, size: usize) -> Self {
        self.config.default_stack_size = size;
        self
    }

    pub fn preemption_interval(mut self, ms: u64) -> Self {
        self.config.preemption_interval_ms = ms;
        self
    }

    pub fn build(self) -> Lachesis {
        Lachesis {
            config: self.config,
            initialized: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
}
