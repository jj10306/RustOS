use alloc::boxed::Box;
use core::time::Duration;

use crate::console::{CONSOLE, kprintln};
use crate::process::{State, Process};
use crate::traps::TrapFrame;
use crate::SCHEDULER;
use crate::param::TICK;
use kernel_api::*;
use pi::timer;


/// Sleep for `ms` milliseconds.
///
/// This system call takes one parameter: the number of milliseconds to sleep.
///
/// In addition to the usual status value, this system call returns one
/// parameter: the approximate true elapsed time from when `sleep` was called to
/// when `sleep` returned.
/// //TODO: Fix bug where next process after slept one wont get full quantum
pub fn sys_sleep(ms: u32, tf: &mut TrapFrame) {
    kprintln!("goodnight sleep.... {}", tf.get_tpidr());
    //TODO: consider weird wrap around cases
    let wake_up_millis = timer::current_time().as_millis() + ms as u128;
    let wake_up_alarm = Duration::from_millis(wake_up_millis as u64);

    let boxed_fnmut = Box::new(move |p: &mut Process| {
        let current_time = timer::current_time();
        // kprintln!("alarm: {:?}, current: {:?}", wake_up_alarm, current_time);
        // kprintln!("Process {:?}, Current: {:?}, Alarm: {:?}",p.context.get_tpidr(), current_time, wake_up_alarm);
        if current_time >= wake_up_alarm {
            // kprintln!("in da cut, {:?}", p.context.get_tpidr());
            true
        } else {
            false
        }
    });
    //change the approximation to be how many processes are ahead of it in the queue

    tf.set_gpr(0, ms as u64 + (TICK.as_millis() as u64 * 5));
    tf.set_gpr(7, 1);
    timer::tick_in(TICK);
    SCHEDULER.switch(State::Waiting(boxed_fnmut), tf);  
}

// maxtimer = 10
// currtime = 8, sleeptime = 4 : target = 2


/// Returns current time.
///
/// This system call does not take parameter.
///
/// In addition to the usual status value, this system call returns two
/// parameter:
///  - current time as seconds
///  - fractional part of the current time, in nanoseconds.
pub fn sys_time(tf: &mut TrapFrame) {
    unimplemented!("sys_time()");
}

/// Kills current process.
///
/// This system call does not take paramer and does not return any value.
pub fn sys_exit(tf: &mut TrapFrame) {
    unimplemented!("sys_exit()");
}

/// Write to console.
///
/// This system call takes one parameter: a u8 character to print.
///
/// It only returns the usual status value.
pub fn sys_write(b: u8, tf: &mut TrapFrame) {
    unimplemented!("sys_write()");
}

/// Returns current process's ID.
///
/// This system call does not take parameter.
///
/// In addition to the usual status value, this system call returns a
/// parameter: the current process's ID.
pub fn sys_getpid(tf: &mut TrapFrame) {
    unimplemented!("sys_getpid()");
}

pub fn handle_syscall(num: u16, tf: &mut TrapFrame) {
    use crate::console::kprintln;
    match num {
        1 => {
            let ms_to_sleep = tf.get_gpr(0);
            //sleep 
            sys_sleep(ms_to_sleep as u32, tf);
        },
        _ => {
            panic!("Other syscalls not yet implemented");
        }
    }
}
