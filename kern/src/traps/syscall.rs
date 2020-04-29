use alloc::boxed::Box;
use core::time::Duration;

use crate::console::{CONSOLE, kprintln, kprint};
use crate::process::{State, Process};
use crate::traps::TrapFrame;
use crate::param::TICK;
use crate::param::USER_IMG_BASE;
use smoltcp::wire::{IpAddress, IpEndpoint};
use crate::{ETHERNET, SCHEDULER};

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
    // kprintln!("goodnight sleep.... {}", tf.get_tpidr());
    kprint!("we sleepy ... \n");
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
    let current_time = timer::current_time();
    let secs = current_time.as_secs();
    let nanos = current_time.as_nanos() as u64;
    tf.set_gpr(0, secs);
    tf.set_gpr(1, nanos);
    tf.set_gpr(7, 1);
}

/// Kills the current process.
///
/// This system call does not take paramer and does not return any value.
pub fn sys_exit(tf: &mut TrapFrame) {
    timer::tick_in(TICK);
    SCHEDULER.switch(State::Dead, tf); 
}

/// Writes to console.
///
/// This system call takes one parameter: a u8 character to print.
///
/// It only returns the usual status value.
pub fn sys_write(b: u8, tf: &mut TrapFrame) {
    kprint!("{}", b as char);
    tf.set_gpr(7, 1);
}

/// Returns the current process's ID.
///
/// This system call does not take parameter.
///
/// In addition to the usual status value, this system call returns a
/// parameter: the current process's ID.
pub fn sys_getpid(tf: &mut TrapFrame) {
    let pid = tf.get_tpidr();
    tf.set_gpr(0, pid);
    tf.set_gpr(7, 1);
}


/// Creates a socket and saves the socket handle in the current process's
/// socket list.
///
/// This function does neither take any parameter nor return anything,
/// except the usual return code that indicates successful syscall execution.
pub fn sys_sock_create(tf: &mut TrapFrame) {
    // Lab 5 2.D
    unimplemented!("sys_sock_create")
}

/// Returns the status of a socket.
///
/// This system call takes a socket descriptor as the first parameter.
///
/// In addition to the usual status value, this system call returns four boolean
/// values that describes the status of the queried socket.
///
/// - x0: is_active
/// - x1: is_listening
/// - x2: can_send
/// - x3: can_recv
///
/// # Errors
/// This function returns `OsError::InvalidSocket` if a socket that corresponds
/// to the provided descriptor is not found.
pub fn sys_sock_status(sock_idx: usize, tf: &mut TrapFrame) {
    // Lab 5 2.D
    unimplemented!("sys_sock_status")
}

/// Connects a local ephemeral port to a remote IP endpoint with a socket.
///
/// This system call takes a socket descriptor as the first parameter, the IP
/// of the remote endpoint as the second paramter in big endian, and the port
/// number of the remote endpoint as the third parameter.
///
/// `handle_syscall` should read the value of registers and create a struct that
/// implements `Into<IpEndpoint>` when calling this function.
///
/// It only returns the usual status value.
///
/// # Errors
/// This function can return following errors:
///
/// - `OsError::NoEntry`: Fails to allocate an ephemeral port
/// - `OsError::InvalidSocket`: Cannot find a socket that corresponds to the provided descriptor.
/// - `OsError::IllegalSocketOperation`: `connect()` returned `smoltcp::Error::Illegal`.
/// - `OsError::BadAddress`: `connect()` returned `smoltcp::Error::Unaddressable`.
/// - `OsError::Unknown`: All the other errors from calling `connect()`.
pub fn sys_sock_connect(
    sock_idx: usize,
    remote_endpoint: impl Into<IpEndpoint>,
    tf: &mut TrapFrame,
) {
    // Lab 5 2.D
    unimplemented!("sys_sock_connect")
}

/// Listens on a local port for an inbound connection.
///
/// This system call takes a socket descriptor as the first parameter and the
/// local ports to listen on as the second parameter.
///
/// It only returns the usual status value.
///
/// # Errors
/// This function can return following errors:
///
/// - `OsError::InvalidSocket`: Cannot find a socket that corresponds to the provided descriptor.
/// - `OsError::IllegalSocketOperation`: `listen()` returned `smoltcp::Error::Illegal`.
/// - `OsError::BadAddress`: `listen()` returned `smoltcp::Error::Unaddressable`.
/// - `OsError::Unknown`: All the other errors from calling `listen()`.
pub fn sys_sock_listen(sock_idx: usize, local_port: u16, tf: &mut TrapFrame) {
    // Lab 5 2.D
    unimplemented!("sys_sock_listen")
}

/// Returns a slice from a virtual address and a legnth.
///
/// # Errors
/// This functions returns `Err(OsError::BadAddress)` if the slice is not entirely
/// in userspace.
unsafe fn to_user_slice<'a>(va: usize, len: usize) -> OsResult<&'a [u8]> {
    let overflow = va.checked_add(len).is_none();
    if va >= USER_IMG_BASE && !overflow {
        Ok(core::slice::from_raw_parts(va as *const u8, len))
    } else {
        Err(OsError::BadAddress)
    }
}
/// Returns a mutable slice from a virtual address and a legnth.
///
/// # Errors
/// This functions returns `Err(OsError::BadAddress)` if the slice is not entirely
/// in userspace.
unsafe fn to_user_slice_mut<'a>(va: usize, len: usize) -> OsResult<&'a mut [u8]> {
    let overflow = va.checked_add(len).is_none();
    if va >= USER_IMG_BASE && !overflow {
        Ok(core::slice::from_raw_parts_mut(va as *mut u8, len))
    } else {
        Err(OsError::BadAddress)
    }
}

/// Sends data with a connected socket.
///
/// This system call takes a socket descriptor as the first parameter, the
/// address of the buffer as the second parameter, and the length of the buffer
/// as the third parameter.
///
/// In addition to the usual status value, this system call returns one
/// parameter: the number of bytes sent.
///
/// # Errors
/// This function can return following errors:
///
/// - `OsError::InvalidSocket`: Cannot find a socket that corresponds to the provided descriptor.
/// - `OsError::BadAddress`: The address and the length pair does not form a valid userspace slice.
/// - `OsError::IllegalSocketOperation`: `send_slice()` returned `smoltcp::Error::Illegal`.
/// - `OsError::Unknown`: All the other errors from smoltcp.
pub fn sys_sock_send(sock_idx: usize, va: usize, len: usize, tf: &mut TrapFrame) {
    // Lab 5 2.D
    unimplemented!("sys_sock_send")
}

/// Receives data from a connected socket.
///
/// This system call takes a socket descriptor as the first parameter, the
/// address of the buffer as the second parameter, and the length of the buffer
/// as the third parameter.
///
/// In addition to the usual status value, this system call returns one
/// parameter: the number of bytes read.
///
/// # Errors
/// This function can return following errors:
///
/// - `OsError::InvalidSocket`: Cannot find a socket that corresponds to the provided descriptor.
/// - `OsError::BadAddress`: The address and the length pair does not form a valid userspace slice.
/// - `OsError::IllegalSocketOperation`: `recv_slice()` returned `smoltcp::Error::Illegal`.
/// - `OsError::Unknown`: All the other errors from smoltcp.
pub fn sys_sock_recv(sock_idx: usize, va: usize, len: usize, tf: &mut TrapFrame) {
    // Lab 5 2.D
    unimplemented!("sys_sock_recv")
}

/// Writes a UTF-8 string to the console.
///
/// This system call takes the address of the buffer as the first parameter and
/// the length of the buffer as the second parameter.
///
/// In addition to the usual status value, this system call returns the length
/// of the UTF-8 message.
///
/// # Errors
/// This function can return following errors:
///
/// - `OsError::BadAddress`: The address and the length pair does not form a valid userspace slice.
/// - `OsError::InvalidArgument`: The provided buffer is not UTF-8 encoded.
pub fn sys_write_str(va: usize, len: usize, tf: &mut TrapFrame) {
    let result = unsafe { to_user_slice(va, len) }
        .and_then(|slice| core::str::from_utf8(slice).map_err(|_| OsError::InvalidArgument));

    match result {
        Ok(msg) => {
            kprint!("{}", msg);

            tf.set_gpr(0, msg.len() as u64);
            tf.set_gpr(7, OsError::Ok as u64);
        }
        Err(e) => {
            tf.set_gpr(7, e as u64);
        }
    }
}

pub fn handle_syscall(num: u16, tf: &mut TrapFrame) {
    match num {
        1 => {
            let ms_to_sleep = tf.get_gpr(0);
            sys_sleep(ms_to_sleep as u32, tf);
        },
        2 => {
            sys_time(tf);
        },
        3 => {
            sys_exit(tf);
        },
        4 => {
            let b = tf.get_gpr(0);
            sys_write(b as u8, tf);
        },
        5 => {
            sys_getpid(tf);
        },
        6 => {
            let slice_ptr_va = tf.get_gpr(0) as usize;
            let slice_len = tf.get_gpr(1) as usize;
            sys_write_str(slice_ptr_va, slice_len, tf);
        }
        _ => {
            panic!("Other syscalls not yet implemented");
        }
    }
}
