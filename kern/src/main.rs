#![feature(alloc_error_handler)]
#![feature(const_fn)]
#![feature(decl_macro)]
#![feature(asm)]
#![feature(global_asm)]
#![feature(optin_builtin_traits)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
mod init;

pub mod console;
pub mod mutex;
pub mod shell;

use console::kprintln;

const GPIO_BASE: usize = 0x3F000000 + 0x200000;

const GPIO_FSEL1: *mut u32 = (GPIO_BASE + 0x04) as *mut u32;
const GPIO_SET0: *mut u32 = (GPIO_BASE + 0x1C) as *mut u32;
const GPIO_CLR0: *mut u32 = (GPIO_BASE + 0x28) as *mut u32;

// FIXME: You need to add dependencies here to
// test your drivers (Phase 2). Add them as needed.
use pi::{timer::spin_sleep, gpio::Gpio, uart::MiniUart};
use core::time::Duration;
use core::fmt::Write;



unsafe fn kmain() -> ! {
    // let mut gpio6_output = Gpio::new(6).into_output();
    // let mut gpio13_output = Gpio::new(13).into_output();
    // let mut gpio19_output = Gpio::new(19).into_output();
    // let mut gpio26_output = Gpio::new(26).into_output();
    // FIXME: Start the shell.

    // *GPIO_FSEL1.write_volatile(GPIO_FSEL1.read_volatile() & ! (0b111 << 18));

    //* GPIO_FSEL1.write_volatile(GPIO_FSEL1.read_volatile() | (0b001 << 18));
    let mut mini_uart = MiniUart::new();
    loop {
        let byte = mini_uart.read_byte();
        mini_uart.write_byte(byte);
    }
}
