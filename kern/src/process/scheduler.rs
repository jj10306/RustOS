use alloc::boxed::Box;
use alloc::collections::vec_deque::VecDeque;
use shim::path::PathBuf;
use alloc::vec::Vec;

use core::ffi::c_void;
use core::fmt;
use core::mem;
use core::time::Duration;

use aarch64::*;
use crate::shell::shell;
use pi::{timer, interrupt, local_interrupt};
use crate::console::{kprintln, kprint};


use crate::param::{PAGE_MASK, PAGE_SIZE, TICK, USER_IMG_BASE, PAGE_ALIGN};
use pi::local_interrupt::LocalInterrupt;
use smoltcp::time::Instant;

use crate::mutex::Mutex;
use crate::net::uspi::TKernelTimerHandle;
use crate::param::*;
use crate::percore::{get_preemptive_counter, is_mmu_ready, local_irq};
use crate::process::{Id, Process, State};
use crate::traps::irq::IrqHandlerRegistry;
use crate::traps::TrapFrame;
use crate::VMM;
use crate::{SCHEDULER, GLOABAL_IRQ};

use crate::{ETHERNET, USB};

/// Process scheduler for the entire machine.
#[derive(Debug)]
pub struct GlobalScheduler(Mutex<Option<Box<Scheduler>>>);

impl GlobalScheduler {
    /// Returns an uninitialized wrapper around a local scheduler.
    pub const fn uninitialized() -> GlobalScheduler {
        GlobalScheduler(Mutex::new(None))
    }

    /// Enters a critical region and execute the provided closure with a mutable
    /// reference to the inner scheduler.
    pub fn critical<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Scheduler) -> R,
    {
        let mut guard = self.0.lock();
        f(guard.as_mut().expect("scheduler uninitialized"))
    }

    /// Adds a process to the scheduler's queue and returns that process's ID.
    /// For more details, see the documentation on `Scheduler::add()`.
    pub fn add(&self, process: Process) -> Option<Id> {
        self.critical(move |scheduler| scheduler.add(process))
    }

    /// Performs a context switch using `tf` by setting the state of the current
    /// process to `new_state`, saving `tf` into the current process, and
    /// restoring the next process's trap frame into `tf`. For more details, see
    /// the documentation on `Scheduler::schedule_out()` and `Scheduler::switch_to()`.
    pub fn switch(&self, new_state: State, tf: &mut TrapFrame) -> Id {
        self.critical(|scheduler| scheduler.schedule_out(new_state, tf));
        self.switch_to(tf)
    }

    /// Loops until it finds the next process to schedule.
    /// Call `wfi()` in the loop when no process is ready.
    /// For more details, see the documentation on `Scheduler::switch_to()`.
    ///
    /// Returns the process's ID when a ready process is found.
    pub fn switch_to(&self, tf: &mut TrapFrame) -> Id {
        loop {
            let rtn = self.critical(|scheduler| scheduler.switch_to(tf));
            if let Some(id) = rtn {
                trace!(
                    "[core-{}] switch_to {:?}, pc: {:x}, lr: {:x}, x29: {:x}, x28: {:x}, x27: {:x}",
                    affinity(),
                    id,
                    tf.get_elr(),
                    tf.get_gpr(30),
                    tf.get_gpr(29),
                    tf.get_gpr(28),
                    tf.get_gpr(27)
                );
                return id;
            }

            aarch64::wfe();
        }
    }

    /// Kills currently running process and returns that process's ID.
    /// For more details, see the documentation on `Scheduler::kill()`.
    #[must_use]
    pub fn kill(&self, tf: &mut TrapFrame) -> Option<Id> {
        self.critical(|scheduler| scheduler.kill(tf))
    }

    pub fn timer_handler(tf: &mut TrapFrame) {
        timer::tick_in(TICK);
        //TODO: should this always be Ready or could it be Waiting? Do I need to add another parameter to know this info
        SCHEDULER.switch(State::Ready, tf);
    }
    pub fn local_timer_handler(tf: &mut TrapFrame) {
        kprint!("local tick on {}\n", affinity());
        local_interrupt::local_tick_in(affinity(), TICK);
        //TODO: should this always be Ready or could it be Waiting? Do I need to add another parameter to know this info
        SCHEDULER.switch(State::Ready, tf);
    }
    /// Starts executing processes in user space using timer interrupt based
    /// preemptive scheduling. This method should not return under normal
    /// conditions.
    pub fn start(&self) -> ! {
        let fake_tf: &mut TrapFrame = &mut Default::default();
        self.switch_to(fake_tf);
        if affinity() == 0 {
            self.initialize_global_timer_interrupt();
        }
        self.initialize_local_timer_interrupt();
        unsafe {
            //must restore x28 - x31 since context restore doesn't restore these registers, give up the SP adjustment
            asm!(
                "
                mov SP, $0
                bl context_restore
                ldp x28, x29, [SP], #16
                ldp lr, xzr, [SP], #16
                eret"
            :: "r"(fake_tf)
            :: "volatile");

        }
        loop{}
    }

    /// # Lab 4
    /// Initializes the global timer interrupt with `pi::timer`. The timer
    /// should be configured in a way that `Timer1` interrupt fires every
    /// `TICK` duration, which is defined in `param.rs`.
    ///
    /// # Lab 5
    /// Registers a timer handler with `Usb::start_kernel_timer` which will
    /// invoke `poll_ethernet` after 1 second.
    pub fn initialize_global_timer_interrupt(&self) {
        // let mut interrupt_controller = interrupt::Controller::new();
        // interrupt_controller.enable(interrupt::Interrupt::Timer1);
        // GLOABAL_IRQ.register(interrupt::Interrupt::Timer1, Box::new(GlobalScheduler::timer_handler));
    }

    /// Initializes the per-core local timer interrupt with `pi::local_interrupt`.
    /// The timer should be configured in a way that `CntpnsIrq` interrupt fires
    /// every `TICK` duration, which is defined in `param.rs`.
    pub fn initialize_local_timer_interrupt(&self) {
        kprint!("in initialize local timer interrupt\n");
        // Lab 5 2.C    
        let mut local_interrupt_controller = local_interrupt::LocalController::new(affinity());
        local_interrupt_controller.enable_local_timer();
        local_irq().register(LocalInterrupt::CNTPNSIRQ, Box::new(GlobalScheduler::local_timer_handler));
        local_interrupt_controller.tick_in(TICK);
    }

    /// Initializes the scheduler and add userspace processes to the Scheduler.
    pub unsafe fn initialize(&self) {
        
        // use pi::interrupt;
        use alloc::boxed::Box;

    

        let sleep_path = &PathBuf::from("/sleep.bin");
        let fib_path = &PathBuf::from("/fib.bin");

        let mut process_0 = Process::load(fib_path).expect("Error creating the process");
        let mut process_1 = Process::load(fib_path).expect("Error creating the process");
        let mut process_2 = Process::load(fib_path).expect("Error creating the process");
        let mut process_3 = Process::load(fib_path).expect("Error creating the process");


        let scheduler = Scheduler::new();
        *self.0.lock() = Some(scheduler);

        self.add(process_0);
        self.add(process_1);
        self.add(process_2);
        self.add(process_3);    
    }

    // The following method may be useful for testing Lab 4 Phase 3:
    //
    // * A method to load a extern function to the user process's page table.
    //
    pub fn test_phase_3(&self, proc: &mut Process){
        use crate::vm::{VirtualAddr, PagePerm};
    
        let mut page = proc.vmap.alloc(
            VirtualAddr::from(USER_IMG_BASE as u64), PagePerm::RWX);
    
        let text = unsafe {
            core::slice::from_raw_parts(test_user_process as *const u8, 24)
        };
    
        page[0..24].copy_from_slice(text);
    }
    
}

/// Poll the ethernet driver and re-register a timer handler using
/// `Usb::start_kernel_timer`.
extern "C" fn poll_ethernet(_: TKernelTimerHandle, _: *mut c_void, _: *mut c_void) {
    // Lab 5 2.B
    unimplemented!("poll_ethernet")
}

/// Internal scheduler struct which is not thread-safe.
pub struct Scheduler {
    processes: VecDeque<Process>,
    last_id: Option<Id>,
}

impl Scheduler {
    /// Returns a new `Scheduler` with an empty queue.
    fn new() -> Box<Scheduler> {
        Box::new(Scheduler { processes: VecDeque::new(), last_id: None })
    }

    /// Adds a process to the scheduler's queue and returns that process's ID if
    /// a new process can be scheduled. The process ID is newly allocated for
    /// the process and saved in its `trap_frame`. If no further processes can
    /// be scheduled, returns `None`.
    ///
    /// It is the caller's responsibility to ensure that the first time `switch`
    /// is called, that process is executing on the CPU.
    fn add(&mut self, mut process: Process) -> Option<Id> {
        if let Some(last_id) = self.last_id {
            if let Some(new_id) = last_id.checked_add(1) {
                process.context.set_tpidr(new_id);
                self.processes.push_back(process);
                self.last_id = Some(new_id);
                self.last_id
            } else {
                None
            }
        } else {
            let new_id = 0;
            process.context.set_tpidr(new_id);
            self.processes.push_back(process);
            self.last_id = Some(new_id);
            self.last_id
        }
    }

    /// Finds the currently running process, sets the current process's state
    /// to `new_state`, prepares the context switch on `tf` by saving `tf`
    /// into the current process, and push the current process back to the
    /// end of `processes` queue.
    ///
    /// If the `processes` queue is empty or there is no current process,
    /// returns `false`. Otherwise, returns `true`.
    fn schedule_out(&mut self, new_state: State, tf: &mut TrapFrame) -> bool {
        // kprintln!("{:?}", self.processes);
        // let mut a = 0;
        // while a < 1000 {
        //     kprintln!("{}", a);
        //     a += 1;
        // }
        if self.processes.is_empty() { return false; }
        let mut i = 0;
        let mut found = false;
        for process in &mut self.processes {
            if let State::Running = process.state {
                if process.context.get_tpidr() == tf.get_tpidr() {
                    found = true;
                    break;
                }
            } 
            i += 1;
        }
        if !found { return false; }
        let mut removed_process = self.processes.remove(i).expect("Index out of bounds when trying to remove");
        removed_process.state = new_state;
        let pid = removed_process.context.get_tpidr();
        removed_process.context = Box::new(*tf);
        self.processes.push_back(removed_process);

        sev();
        true
    }

    /// Finds the next process to switch to, brings the next process to the
    /// front of the `processes` queue, changes the next process's state to
    /// `Running`, and performs context switch by restoring the next process`s
    /// trap frame into `tf`.
    ///
    /// If there is no process to switch to, returns `None`. Otherwise, returns
    /// `Some` of the next process`s process ID.
    fn switch_to(&mut self, tf: &mut TrapFrame) -> Option<Id> {
        // kprintln!("{:?}", self.processes);
        let mut i = 0;
        let mut found = false;
        // kprintln!("Before queue: {:?}\n\n\n\n\n", &self.processes);

        for process in &mut self.processes {
            if process.is_ready() {
                found = true;
                break;
            }
            i += 1;
        }
        if !found { return None; }
        // TODO: Cleaner way of doing this
        //temporarily remove selected process from queue so we can mutate it before we add it to the front
        let mut ready_process = self.processes.remove(i).expect("Index out of bounds when trying to remove");
        ready_process.state = State::Running;
        let context_to_restore = *ready_process.context;
        // restore the process' context from what is saved
        *tf = context_to_restore;
        // kprintln!("{:?}", context_to_restore);
        let ready_pid = tf.get_tpidr();
        // add the 'Running' queue to the front of the queue now that its context has been restored
        self.processes.push_front(ready_process);
        // panic!("{}", ready_pid);
        // kprintln!("After queue: {:?}\n\n\n\n\n", &self.processes);
        Some(ready_pid)


    }

    /// Kills currently running process by scheduling out the current process
    /// as `Dead` state. Releases all process resources held by the process,
    /// removes the dead process from the queue, drops the dead process's
    /// instance, and returns the dead process's process ID.
    fn kill(&mut self, tf: &mut TrapFrame) -> Option<Id> {
        if let true = self.schedule_out(State::Dead, tf) {
            let process_to_be_dropped = self.processes.pop_back();
            drop(process_to_be_dropped);
            Some(tf.get_tpidr())
        } else {
            None
        }
    }

    /// Releases all process resources held by the current process such as sockets.
    fn release_process_resources(&mut self, tf: &mut TrapFrame) {
        // Lab 5 2.C
        unimplemented!("release_process_resources")
    }

    /// Finds a process corresponding with tpidr saved in a trap frame.
    /// Panics if the search fails.
    pub fn find_process(&mut self, tf: &TrapFrame) -> &mut Process {
        for i in 0..self.processes.len() {
            if self.processes[i].context.get_tpidr() == tf.get_tpidr() {
                return &mut self.processes[i];
            }
        }
        panic!("Invalid TrapFrame");
    }
}

impl fmt::Debug for Scheduler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let len = self.processes.len();
        write!(f, "  [Scheduler] {} processes in the queue\n", len)?;
        for i in 0..len {
            write!(
                f,
                "    queue[{}]: proc({:3})-{:?} \n",
                i, self.processes[i].context.get_tpidr(), self.processes[i].state
            )?;
        }
        Ok(())
    }
}

pub extern "C" fn  test_user_process() -> ! {
    loop {
        let ms = 10000;
        let error: u64;
        let elapsed_ms: u64;

        unsafe {
            asm!("mov x0, $2
              svc 1
              mov $0, x0
              mov $1, x7"
                 : "=r"(elapsed_ms), "=r"(error)
                 : "r"(ms)
                 : "x0", "x7"
                 : "volatile");
        }
    }
}
