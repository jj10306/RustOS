#![feature(asm)]
#![no_std]
#![no_main]

mod cr0;

use kernel_api::{println, print};
use kernel_api::syscall::{getpid, time, write};

fn fib(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fib(n - 1) + fib(n - 2),
    }
}

fn main() {
    // println!("[PID: {}](Time = {:?}) Started...", getpid(), time());
    print!("Started: {:?}\n", time());
    // print!("Fib started \n");
    let rtn = fib(40);
    print!("Ended: {:?}\n", time());
    // print!("Fib ended \n");
    // println!("[PID: {}](Time = {:?}) Ended: Result = {}", getpid(), time(), rtn);

}
