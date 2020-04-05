use alloc::boxed::Box;
use alloc::vec::Vec;
use shim::io;
use shim::path::Path;

use aarch64::*;

use crate::param::*;
use crate::process::{Stack, State};
use crate::traps::TrapFrame;
use crate::vm::*;
use crate::fs::PiVFatHandle;
use crate::console::kprintln;

use crate::FILESYSTEM;
use fat32::traits::{FileSystem, Entry};

use kernel_api::{OsError, OsResult};

use fat32::vfat::{File};
use shim::io::Read;


/// Type alias for the type of a process ID.
pub type Id = u64;

/// A structure that represents the complete state of a process.
#[derive(Debug)]
pub struct Process {
    /// The saved trap frame of a process.
    pub context: Box<TrapFrame>,
    /// The memory allocation used for the process's stack.
    pub stack: Stack,
    /// The page table describing the Virtual Memory of the process
    pub vmap: Box<UserPageTable>,
    /// The scheduling state of the process.
    pub state: State,
}

impl Process {
    /// Creates a new process with a zeroed `TrapFrame` (the default), a zeroed
    /// stack of the default size, and a state of `Ready`.
    ///
    /// If enough memory could not be allocated to start the process, returns
    /// `None`. Otherwise returns `Some` of the new `Process`.
    pub fn new() -> OsResult<Process> {
        let trap_frame: TrapFrame = Default::default();
        let stack = Stack::new();
        let vmap = Box::new(UserPageTable::new());
        if stack.is_none() { 
            Err(OsError::NoMemory)
        } else {
            let stack = stack.unwrap();
            let state = State::Ready;
            let context = Box::new(trap_frame);
            let process = Process { context, stack, vmap, state };
    
            Ok(process)
        }

    }

    /// Load a program stored in the given path by calling `do_load()` method.
    /// Set trapframe `context` corresponding to the its page table.
    /// `sp` - the address of stack top
    /// `elr` - the address of image base.
    /// `ttbr0` - the base address of kernel page table
    /// `ttbr1` - the base address of user page table
    /// `spsr` - `F`, `A`, `D` bit should be set.
    ///
    /// Returns Os Error if do_load fails.
    pub fn load<P: AsRef<Path>>(pn: P) -> OsResult<Process> {
        use crate::VMM;

        let mut process = Process::do_load(pn)?;

        process.context.set_elr(USER_IMG_BASE as u64);
        // panic!("{}", Process::get_stack_top().as_u64());
        process.context.set_sp(Process::get_stack_top().as_u64());
        //set the EL to be 0 so that on 'eret' the process is running in EL0
        process.context.set_spsr(0x0000_0340);
        //set the IRq interrupt bit to 0 so that it will take the timer interrupt for scheduling
        // process.context.set_spsr(process.context.get_spsr() & !(0b1 << 7));
        // process.context.set_spsr(process.context.get_spsr() & !(0b1 << 7));
        // set ttbr0 to base address of kernel page table
        process.context.set_ttbr0(VMM.get_baddr().as_u64());
        // set ttbr1 to base address of user page table
        process.context.set_ttbr1(process.vmap.get_baddr().as_u64());
        
        process.state = State::Ready;


        Ok(process)
    }

    /// Creates a process and open a file with given path.
    /// Allocates one page for stack with read/write permission, and N pages with read/write/execute
    /// permission to load file's contents.
    fn do_load<P: AsRef<Path>>(pn: P) -> OsResult<Process> {
        match FILESYSTEM.open(pn) {
            Ok(entry) => {
                match entry.into_file() {
                    Some(file) => Ok(Process::create_process_from_file(file)?),
                    None => { Err(OsError::InvalidArgument) }
                }
            },
            Err(e) => { Err(OsError::NoEntry) }
        }

    }

    fn create_process_from_file(mut file: File<PiVFatHandle>) -> OsResult<Process> {
    
        let mut process = Process::new().unwrap();
        //alloacte stack in USER virtual memory space
        let stack_page = process.vmap.alloc(Process::get_stack_base(), PagePerm::RW);

        let mut page_start_address = USER_IMG_BASE as u64;
        let mut bytes_copied_to_mem = 0;
        let mut iterations = 0;
        while bytes_copied_to_mem < file.size {
            let mut page = process.vmap.alloc(VirtualAddr::from(page_start_address), PagePerm::RWX);
            bytes_copied_to_mem += file.read(page)?;
            page_start_address += PAGE_SIZE as u64;
            // kprintln!("bytes copied to mem: {}", bytes_copied_to_mem);
            // kprintln!("Page bits {:?}", &page[..24]);
            iterations += 1;
        }

        Ok(process)        
    }
    /// Returns the highest `VirtualAddr` that is supported by this system.
    pub fn get_max_va() -> VirtualAddr {
        VirtualAddr::from(core::usize::MAX)
    }

    /// Returns the `VirtualAddr` represents the base address of the user
    /// memory space.
    pub fn get_image_base() -> VirtualAddr {
        VirtualAddr::from(USER_IMG_BASE)
    }

    /// Returns the `VirtualAddr` represents the base address of the user
    /// process's stack.
    pub fn get_stack_base() -> VirtualAddr {
        VirtualAddr::from(USER_STACK_BASE)
    }

    /// Returns the `VirtualAddr` represents the top of the user process's
    /// stack.
    pub fn get_stack_top() -> VirtualAddr {
        let base = Process::get_stack_base().as_usize();

        let top = base + PAGE_SIZE - 1;
        //stack pointer must be 16 byte aligned on ARM
        let align = 16;
        let divides = top / align;
        if divides > 0 {
            VirtualAddr::from(align * divides)
        } else {
            VirtualAddr::from(0)
        }
    }

    /// Returns `true` if this process is ready to be scheduled.
    ///
    /// This functions returns `true` only if one of the following holds:
    ///
    ///   * The state is currently `Ready`.
    ///
    ///   * An event being waited for has arrived.
    ///
    ///     If the process is currently waiting, the corresponding event
    ///     function is polled to determine if the event being waiting for has
    ///     occured. If it has, the state is switched to `Ready` and this
    ///     function returns `true`.
    ///
    /// Returns `false` in all other cases.
    pub fn is_ready(&mut self) -> bool {
        match self.state {
            State::Ready => true,
            State::Waiting(_) => {
                use core::mem::replace;
                let mut owned_state = replace(&mut self.state, State::Ready);
                if let State::Waiting(mut poll_fn) = owned_state {
                    if poll_fn(self) == true {
                        true
                    } else {
                        replace(&mut self.state, State::Waiting(poll_fn));
                        false
                    }
                } else {
                    panic!("You shouldn't be here");
                }
            },
            _ => false
        }
    }

}