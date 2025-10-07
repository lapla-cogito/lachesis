pub const PAGE_SIZE: usize = 4 * 1024; // 4KiB

#[cfg(target_arch = "x86_64")]
#[repr(C)]
pub struct Registers {
    pub rbx: u64,
    pub rbp: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rsp: u64,
    pub rdx: u64,
}

#[cfg(target_arch = "aarch64")]
#[repr(C)]
pub struct Registers {
    // Floating-point registers d8-d15 (each pair is 16 bytes)
    pub d8_d9: [u64; 2],
    pub d10_d11: [u64; 2],
    pub d12_d13: [u64; 2],
    pub d14_d15: [u64; 2],
    // General-purpose registers x19-x28 (each pair is 16 bytes)
    pub x19_x20: [u64; 2],
    pub x21_x22: [u64; 2],
    pub x23_x24: [u64; 2],
    pub x25_x26: [u64; 2],
    pub x27_x28: [u64; 2],
    // Link register and stack pointer (16 bytes)
    pub x30_sp: [u64; 2], // [x30 (link register), sp (stack pointer)]
}

#[cfg(target_arch = "x86_64")]
impl Registers {
    pub fn new(rsp: u64) -> Self {
        unsafe extern "C" {
            fn entry_point() -> !;
        }

        Registers {
            rbx: 0,
            rbp: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rsp,
            rdx: entry_point as usize as u64,
        }
    }
}

#[cfg(target_arch = "aarch64")]
impl Registers {
    pub fn new(sp: u64) -> Self {
        unsafe extern "C" {
            fn entry_point() -> !;
        }

        Registers {
            d8_d9: [0, 0],
            d10_d11: [0, 0],
            d12_d13: [0, 0],
            d14_d15: [0, 0],
            x19_x20: [0, 0],
            x21_x22: [0, 0],
            x23_x24: [0, 0],
            x25_x26: [0, 0],
            x27_x28: [0, 0],
            x30_sp: [entry_point as usize as u64, sp],
        }
    }
}

unsafe extern "C" {
    pub fn set_context(ctx: *mut Registers) -> u64;
    pub fn switch_context(ctx: *const Registers) -> !;
}

pub struct Context {
    pub regs: Registers,
    pub stack: *mut u8,
    pub stack_layout: std::alloc::Layout,
    pub entry: Option<crate::types::Entry>,
    pub id: u64,
    pub state: crate::types::ThreadState,
    pub executable: Option<Box<dyn crate::types::Executable>>,
}

impl Context {
    pub fn get_regs_mut(&mut self) -> *mut Registers {
        &mut self.regs as *mut Registers
    }

    pub fn get_regs(&self) -> *const Registers {
        &self.regs as *const Registers
    }

    pub fn new(func: Option<crate::types::Entry>, stack_size: usize, id: u64) -> Self {
        let layout = std::alloc::Layout::from_size_align(stack_size, PAGE_SIZE).unwrap();
        let stack = unsafe { std::alloc::alloc(layout) };

        // set up guard page for stack overflow protection
        unsafe {
            let non_null_ptr = std::ptr::NonNull::new(stack as *mut std::ffi::c_void).unwrap();
            nix::sys::mman::mprotect(
                non_null_ptr,
                PAGE_SIZE,
                nix::sys::mman::ProtFlags::PROT_NONE,
            )
            .unwrap();
        };

        let regs = Registers::new(stack as u64 + stack_size as u64);

        Context {
            regs,
            stack,
            stack_layout: layout,
            entry: func,
            id,
            state: crate::ThreadState::Ready,
            executable: None,
        }
    }
}
