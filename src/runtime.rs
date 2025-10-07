pub static mut CTX_MAIN: std::option::Option<std::boxed::Box<crate::context::Registers>> =
    std::option::Option::None;
pub static mut UNUSED_STACK: (*mut u8, std::alloc::Layout) =
    (std::ptr::null_mut(), std::alloc::Layout::new::<u8>());
pub static mut CONTEXTS: std::collections::LinkedList<std::boxed::Box<crate::context::Context>> =
    std::collections::LinkedList::new();
pub static mut ID: *mut std::collections::HashSet<u64> = std::ptr::null_mut();
pub static mut CURRENT_THREAD_ID: u64 = 0;

thread_local! {
    static CURRENT_FUNCTION: std::cell::RefCell<Option<Box<dyn crate::types::Executable>>> = std::cell::RefCell::new(None);
}

pub fn get_id() -> u64 {
    loop {
        let rnd = rand::random::<u64>();
        unsafe {
            let id_ptr = &raw mut ID;
            if !(**id_ptr).contains(&rnd) {
                (**id_ptr).insert(rnd);
                return rnd;
            }
        }
    }
}

pub fn execute_main<F>(wrapper: F, stack_size: usize, preemption_interval: u64)
where
    F: FnOnce() + Send + 'static,
{
    CURRENT_FUNCTION.with(|f| {
        *f.borrow_mut() = Some(Box::new(wrapper));
    });

    fn main_entry() {
        CURRENT_FUNCTION.with(|f| {
            if let Some(func) = f.borrow_mut().take() {
                func.execute();
            }
        });
    }

    spawn_from_main(main_entry, stack_size, preemption_interval);
}

pub fn spawn<F>(func: F, stack_size: usize) -> u64
where
    F: FnOnce() + Send + 'static,
{
    unsafe {
        let id = get_id();

        let mut ctx = Box::new(crate::context::Context::new(None, stack_size, id));
        ctx.executable = Some(Box::new(func));

        ctx.state = crate::ThreadState::Ready;
        let contexts_ptr = &raw mut CONTEXTS;
        (*contexts_ptr).push_back(ctx);
        schedule();
        id
    }
}

pub fn schedule() {
    unsafe {
        let contexts_ptr = &raw mut CONTEXTS;
        if (*contexts_ptr).len() <= 1 {
            return;
        }

        let mut ctx = (*contexts_ptr).pop_front().unwrap();
        ctx.state = crate::ThreadState::Ready;
        let regs = ctx.get_regs_mut();
        (*contexts_ptr).push_back(ctx);

        if crate::context::set_context(regs) == 0
            && let Some(next) = (*contexts_ptr).front_mut()
        {
            next.state = crate::ThreadState::Running;
            let current_id_ptr = &raw mut CURRENT_THREAD_ID;
            *current_id_ptr = next.id;
            crate::context::switch_context(next.get_regs());
        }
    }
}

// entry point for green threads
#[unsafe(no_mangle)]
pub extern "C" fn entry_point() -> ! {
    unsafe {
        let contexts_ptr = &raw mut CONTEXTS;
        let ctx = (*contexts_ptr).front_mut().unwrap();

        if let Some(executable) = ctx.executable.take() {
            executable.execute();
        } else if let Some(entry) = ctx.entry {
            entry();
        }

        // Thread cleanup
        let contexts_ptr = &raw mut CONTEXTS;
        let mut ctx = (*contexts_ptr).pop_front().unwrap();
        ctx.state = crate::ThreadState::Terminated;

        // Remove thread ID
        let id_ptr = &raw const ID;
        if !(*id_ptr).is_null() {
            let id_ptr = &raw mut ID;
            (**id_ptr).remove(&ctx.id);
        }

        let unused_ptr = &raw mut UNUSED_STACK;
        *unused_ptr = (ctx.stack, ctx.stack_layout);

        match (*contexts_ptr).front_mut() {
            Some(c) => {
                c.state = crate::ThreadState::Running;
                let current_id_ptr = &raw mut CURRENT_THREAD_ID;
                *current_id_ptr = c.id;
                crate::context::switch_context(c.get_regs());
            }
            None => {
                // All threads finished - return to main
                crate::timer::disable_preemption();
                let ctx_main_ptr = &raw const CTX_MAIN;
                if let Some(c) = &*ctx_main_ptr {
                    crate::context::switch_context(&**c as *const crate::context::Registers);
                }
            }
        };
    }

    unreachable!();
}

pub fn spawn_from_main(func: crate::types::Entry, stack_size: usize, preemption_interval: u64) {
    unsafe {
        let ctx_main_ptr = &raw const CTX_MAIN;
        if (*ctx_main_ptr).is_some() {
            panic!("spawn_from_main is called twice");
        }

        let ctx_main_ptr = &raw mut CTX_MAIN;
        *ctx_main_ptr = Some(Box::new(crate::context::Registers::new(0)));

        if let Some(ctx) = &mut *ctx_main_ptr {
            let mut ids = std::collections::HashSet::new();
            let id_ptr = &raw mut ID;
            *id_ptr = &mut ids as *mut std::collections::HashSet<u64>;

            crate::enable_preemption_with_interval(preemption_interval);

            if crate::context::set_context(&mut **ctx as *mut crate::context::Registers) == 0 {
                let mut first_ctx = Box::new(crate::context::Context::new(
                    Some(func),
                    stack_size,
                    get_id(),
                ));
                first_ctx.state = crate::ThreadState::Running;
                let current_id_ptr = &raw mut CURRENT_THREAD_ID;
                *current_id_ptr = first_ctx.id;
                let contexts_ptr = &raw mut CONTEXTS;
                (*contexts_ptr).push_back(first_ctx);

                let first = (*contexts_ptr).front().unwrap();
                crate::context::switch_context(first.get_regs());
            }

            crate::timer::disable_preemption();

            let ctx_main_ptr = &raw mut CTX_MAIN;
            *ctx_main_ptr = None;
            let id_ptr = &raw mut ID;
            *id_ptr = std::ptr::null_mut();

            // Clear local collections
            ids.clear();

            let unused_ptr = &raw mut UNUSED_STACK;
            *unused_ptr = (std::ptr::null_mut(), std::alloc::Layout::new::<u8>());
        }
    }
}
