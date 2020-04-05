use alloc::boxed::Box;
use alloc::collections::vec_deque::VecDeque;
use shim::path::PathBuf;
use core::fmt;

use aarch64::*;
use crate::shell::shell;
use pi::{timer, interrupt};
use crate::console::{kprintln, kprint};

use crate::mutex::Mutex;
use crate::param::{PAGE_MASK, PAGE_SIZE, TICK, USER_IMG_BASE, PAGE_ALIGN};
use crate::process::{Id, Process, State};
use crate::traps::TrapFrame;
use crate::VMM;
use crate:: IRQ;
use crate::SCHEDULER;


/// Process scheduler for the entire machine.
#[derive(Debug)]
pub struct GlobalScheduler(Mutex<Option<Scheduler>>);

impl GlobalScheduler {
    /// Returns an uninitialized wrapper around a local scheduler.
    pub const fn uninitialized() -> GlobalScheduler {
        GlobalScheduler(Mutex::new(None))
    }

    /// Enter a critical region and execute the provided closure with the
    /// internal scheduler.
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

    pub fn switch_to(&self, tf: &mut TrapFrame) -> Id {
        loop {
            let rtn = self.critical(|scheduler| scheduler.switch_to(tf));
            if let Some(id) = rtn {
                return id;
            }
            // kprintln!("before wifi");
            aarch64::wfi();

            // kprintln!("after wifi");
            // kprintln!("under wfi");
        }
    }

    /// Kills currently running process and returns that process's ID.
    /// For more details, see the documentaion on `Scheduler::kill()`.
    #[must_use]
    pub fn kill(&self, tf: &mut TrapFrame) -> Option<Id> {
        self.critical(|scheduler| scheduler.kill(tf))
    }

    pub fn timer_handler(tf: &mut TrapFrame) {
        timer::tick_in(TICK);
        //TODO: should this always be Ready or could it be Waiting? Do I need to add another parameter to know this info
        SCHEDULER.switch(State::Ready, tf);
    }
    /// Starts executing processes in user space using timer interrupt based
    /// preemptive scheduling. This method should not return under normal conditions.
    pub fn start(&self) -> ! {
        let fake_tf: &mut TrapFrame = &mut Default::default();
        self.switch_to(fake_tf);
        timer::tick_in(TICK);
        unsafe {
            //set SP to the trap frame so when we call context_restore, it is pointing to the correct values
            // asm!(
            //     "
            //     mov SP, $0
            //     bl context_restore
            //     mov x0, SP
            //     add x0, x0, $1
            //     and x0, x0, $2
            //     mov SP, x0
            //     mov x0, xzr
            //     eret"
            // :: "r"(fake_tf), "i"(PAGE_SIZE), "i"(!(PAGE_ALIGN - 1))
            // :: "volatile");
            asm!(
                "
                mov SP, $0
                bl context_restore
                mov x0, SP
                adr x0, _start
                mov SP, x0
                mov x0, xzr
                eret"
            :: "r"(fake_tf)
            :: "volatile");

        }


    

        loop{}
    }

    #[no_mangle]
    pub extern "C" fn start_shell_0() {
        loop { shell("0> "); }
    }
    #[no_mangle]
    pub extern "C" fn start_shell_1() {
        loop { shell("1> "); }
    }
    #[no_mangle]
    pub extern "C" fn start_shell_2() {
        loop { shell("2> "); }
    }
    #[no_mangle]
    pub extern "C" fn start_shell_3() {
        loop { shell("3> "); }
    }
    #[no_mangle]
    pub extern "C" fn start_shell_4() {
        loop { shell("4> "); }
    }
    /// Initializes the scheduler and add userspace processes to the Scheduler
    pub unsafe fn initialize(&self) {
        
        // use pi::interrupt;
        use alloc::boxed::Box;

        let mut interrupt_controller = interrupt::Controller::new();
        interrupt_controller.enable(interrupt::Interrupt::Timer1);
        IRQ.register(interrupt::Interrupt::Timer1, Box::new(GlobalScheduler::timer_handler));
    
        // kprintln!("in initiailize");
        let mut process_0 = self.create_process_0();
        let mut process_1 = self.create_process_1();
        let mut process_2 = self.create_process_2();
        let mut process_3 = self.create_process_3();
        // let mut process_4 = self.create_process_4();
        // let sleep_path = &PathBuf::from("/sleep.bin");

        // let mut process_0 = Process::load(sleep_path).expect("Error creating the process");
        // let mut process_1 = Process::load(sleep_path).expect("Error creating the process");
        // let mut process_2 = Process::load(sleep_path).expect("Error creating the process");
        // let mut process_3 = Process::load(sleep_path).expect("Error creating the process");


        let scheduler = Scheduler::new();
        *self.0.lock() = Some(scheduler);

        // panic!("\n{:?}\n,{:?}\n{:?}\n", process_0, process_1, process_2);
        self.add(process_0);
        self.add(process_1);
        self.add(process_2);

        // self.add(process_3);    
        // self.add(process_4);
    }

    // The following method may be useful for testing Phase 3:
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
    
    //TODO: refactor this
    pub fn create_process_0(&self) -> Process {
        let mut process = Process::new().expect("Error occurred in creating process");
        //set elr to start_shel function so on return from exception it executes this function
        process.context.set_elr(USER_IMG_BASE as u64);
        //set SP_EL1 to the top of the stack so that EL1 handlers have the entire stack space
        process.context.set_sp(process.stack.top().as_u64());
        //set the EL to be 0 so that on 'eret' the process is running in EL0
        // process.context.set_spsr(process.context.get_spsr() & !(0b11 << 2));
        //set the IRq interrupt bit to 0 so that it will take the timer interrupt for scheduling
        // process.context.set_spsr(process.context.get_spsr() & !(0b1 << 7));
        process.context.set_spsr(0x0000_0340);
        // set ttbr0 to base address of kernel page table
        process.context.set_ttbr0(VMM.get_baddr().as_u64());
        // set ttbr1 to base address of user page table
        process.context.set_ttbr1(process.vmap.get_baddr().as_u64());
        
        self.test_phase_3(&mut process);
        //set the State to Ready
        process.state = State::Ready;

        process
    }
    pub fn create_process_1(&self) -> Process {
        let mut process = Process::new().expect("Error occurred in creating process");
        //set elr to start_shel function so on return from exception it executes this function
        process.context.set_elr(USER_IMG_BASE as u64);
        //set SP_EL1 to the top of the stack so that EL1 handlers have the entire stack space
        process.context.set_sp(process.stack.top().as_u64());
        //set the EL to be 0 so that on 'eret' the process is running in EL0
        // process.context.set_spsr(process.context.get_spsr() & !(0b11 << 2));
        //set the IRq interrupt bit to 0 so that it will take the timer interrupt for scheduling
        // process.context.set_spsr(process.context.get_spsr() & !(0b1 << 7));
        process.context.set_spsr(0x0000_0340);
        // set ttbr0 to base address of kernel page table
        process.context.set_ttbr0(VMM.get_baddr().as_u64());
        // set ttbr1 to base address of user page table
        process.context.set_ttbr1(process.vmap.get_baddr().as_u64());
        
        self.test_phase_3(&mut process);
        //set the State to Ready
        process.state = State::Ready;

        process
    }
    pub fn create_process_2(&self) -> Process {
        let mut process = Process::new().expect("Error occurred in creating process");
        //set elr to start_shel function so on return from exception it executes this function
        process.context.set_elr(USER_IMG_BASE as u64);
        //set SP_EL1 to the top of the stack so that EL1 handlers have the entire stack space
        process.context.set_sp(process.stack.top().as_u64());
        //set the EL to be 0 so that on 'eret' the process is running in EL0
        // process.context.set_spsr(process.context.get_spsr() & !(0b11 << 2));
        //set the IRq interrupt bit to 0 so that it will take the timer interrupt for scheduling
        // process.context.set_spsr(process.context.get_spsr() & !(0b1 << 7));
        process.context.set_spsr(0x0000_0340);
        // set ttbr0 to base address of kernel page table
        process.context.set_ttbr0(VMM.get_baddr().as_u64());
        // set ttbr1 to base address of user page table
        process.context.set_ttbr1(process.vmap.get_baddr().as_u64());
        
        self.test_phase_3(&mut process);
        //set the State to Ready
        process.state = State::Ready;

        process
    }
    pub fn create_process_3(&self) -> Process {
        let mut process = Process::new().expect("Error occurred in creating process");
        //set elr to start_shel function so on return from exception it executes this function
        process.context.set_elr(USER_IMG_BASE as u64);
        //set SP_EL1 to the top of the stack so that EL1 handlers have the entire stack space
        process.context.set_sp(process.stack.top().as_u64());
        //set the EL to be 0 so that on 'eret' the process is running in EL0
        // process.context.set_spsr(process.context.get_spsr() & !(0b11 << 2));
        //set the IRq interrupt bit to 0 so that it will take the timer interrupt for scheduling
        // process.context.set_spsr(process.context.get_spsr() & !(0b1 << 7));
        process.context.set_spsr(0x0000_0340);
        // set ttbr0 to base address of kernel page table
        process.context.set_ttbr0(VMM.get_baddr().as_u64());
        // set ttbr1 to base address of user page table
        process.context.set_ttbr1(process.vmap.get_baddr().as_u64());
        
        self.test_phase_3(&mut process);
        //set the State to Ready
        process.state = State::Ready;

        process
    }
    pub fn create_process_4(&self) -> Process {
        let mut process = Process::new().expect("Error occurred in creating process");
        //set elr to start_shel function so on return from exception it executes this function
        process.context.set_elr(USER_IMG_BASE as u64);
        //set SP_EL1 to the top of the stack so that EL1 handlers have the entire stack space
        process.context.set_sp(process.stack.top().as_u64());
        //set the EL to be 0 so that on 'eret' the process is running in EL0
        // process.context.set_spsr(process.context.get_spsr() & !(0b11 << 2));
        //set the IRq interrupt bit to 0 so that it will take the timer interrupt for scheduling
        // process.context.set_spsr(process.context.get_spsr() & !(0b1 << 7));
        process.context.set_spsr(0x0000_0340);
        // set ttbr0 to base address of kernel page table
        process.context.set_ttbr0(VMM.get_baddr().as_u64());
        // set ttbr1 to base address of user page table
        process.context.set_ttbr1(process.vmap.get_baddr().as_u64());
        
        self.test_phase_3(&mut process);
        //set the State to Ready
        process.state = State::Ready;

        process
    }
}

#[derive(Debug)]
pub struct Scheduler {
    processes: VecDeque<Process>,
    last_id: Option<Id>,
}

impl Scheduler {
    /// Returns a new `Scheduler` with an empty queue.
    fn new() -> Scheduler {
        Scheduler { processes: VecDeque::new(), last_id: None } 
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

        // for p_process in & self.processes {
        //     kprintln!("\n{:?}\n", p_process);
        // }

        // timer::spin_sleep(core::time::Duration::from_secs(30));
        // brk!(1);

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
    /// as `Dead` state. Removes the dead process from the queue, drop the
    /// dead process's instance, and returns the dead process's process ID.
    fn kill(&mut self, tf: &mut TrapFrame) -> Option<Id> {
        if let true = self.schedule_out(State::Dead, tf) {
            let process_to_be_dropped = self.processes.pop_back();
            drop(process_to_be_dropped);
            Some(tf.get_tpidr())
        } else {
            None
        }
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

