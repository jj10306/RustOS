#![feature(alloc_error_handler)]
#![feature(const_fn)]
#![feature(decl_macro)]
#![feature(asm)]
#![feature(global_asm)]
#![feature(optin_builtin_traits)]
#![feature(raw_vec_internals)]
#![feature(panic_info_message)]
#![feature(ptr_internals)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
mod init;

extern crate alloc;
#[macro_use]
extern crate log;

pub mod allocator;
pub mod console;
pub mod fs;
pub mod logger;
pub mod mutex;
pub mod net;
pub mod param;
pub mod percore;
pub mod process;
pub mod shell;
pub mod traps;
pub mod vm;



// const GPIO_BASE: usize = 0x3F000000 + 0x200000;

// const GPIO_FSEL1: *mut u32 = (GPIO_BASE + 0x04) as *mut u32;
// const GPIO_SET0: *mut u32 = (GPIO_BASE + 0x1C) as *mut u32;
// const GPIO_CLR0: *mut u32 = (GPIO_BASE + 0x28) as *mut u32;


use pi::{timer::spin_sleep, gpio::Gpio, uart::MiniUart};
use core::time::Duration;
use core::fmt::Write;
use shell::shell;
use console::kprintln;
use aarch64::*;


use pi::atags::Atags;

use allocator::Allocator;
use fs::FileSystem;
use net::uspi::Usb;
use net::GlobalEthernetDriver;
use process::GlobalScheduler;
use traps::irq::{Fiq, GlobalIrq};
use vm::VMManager;

use alloc::vec::Vec;
use alloc::string::String;

#[cfg_attr(not(test), global_allocator)]
pub static ALLOCATOR: Allocator = Allocator::uninitialized();
pub static FILESYSTEM: FileSystem = FileSystem::uninitialized();


pub static SCHEDULER: GlobalScheduler = GlobalScheduler::uninitialized();
pub static VMM: VMManager = VMManager::uninitialized();
pub static USB: Usb = Usb::uninitialized();
pub static GLOABAL_IRQ: GlobalIrq = GlobalIrq::new();
pub static FIQ: Fiq = Fiq::new();
pub static ETHERNET: GlobalEthernetDriver = GlobalEthernetDriver::uninitialized();

extern "C" {
    static __text_beg: u64;
    static __text_end: u64;
    static __bss_beg: u64;
    static __bss_end: u64;
}


fn kmain() -> ! {
    spin_sleep(Duration::from_millis(5000));


    
    unsafe {
        crate::logger::init_logger();

        info!(
            "text beg: {:016x}, end: {:016x}",
            &__text_beg as *const _ as u64, &__text_end as *const _ as u64
        );
        info!(
            "bss  beg: {:016x}, end: {:016x}",
            &__bss_beg as *const _ as u64, &__bss_end as *const _ as u64
        );
        ALLOCATOR.initialize();
        FILESYSTEM.initialize();
        VMM.initialize();
        SCHEDULER.initialize();
        init::initialize_app_cores();
        // kprintln!("yo");
        VMM.wait();
        SCHEDULER.start()
    }
}



















// #![feature(alloc_error_handler)]
// #![feature(const_fn)]
// #![feature(decl_macro)]
// #![feature(asm)]
// #![feature(global_asm)]
// #![feature(optin_builtin_traits)]
// #![feature(ptr_internals)]
// #![feature(raw_vec_internals)]
// #![cfg_attr(not(test), no_std)]
// #![cfg_attr(not(test), no_main)]
// #![feature(panic_info_message)]

// #[cfg(not(test))]
// mod init;

// extern crate alloc;
// #[macro_use]
// extern crate log;

// pub mod allocator;
// pub mod console;
// pub mod fs;
// pub mod logger;
// pub mod mutex;
// pub mod net;
// pub mod param;
// pub mod percore;
// pub mod process;
// pub mod shell;
// pub mod traps;
// pub mod vm;

// use allocator::Allocator;
// use fs::FileSystem;
// use net::uspi::Usb;
// use net::GlobalEthernetDriver;
// use process::GlobalScheduler;
// use traps::irq::{Fiq, GlobalIrq};
// use vm::VMManager;

// use pi::timer::spin_sleep;
// use core::time::Duration;
// use aarch64;
// use console::kprintln;

// #[cfg_attr(not(test), global_allocator)]
// pub static ALLOCATOR: Allocator = Allocator::uninitialized();
// pub static FILESYSTEM: FileSystem = FileSystem::uninitialized();
// pub static SCHEDULER: GlobalScheduler = GlobalScheduler::uninitialized();
// pub static VMM: VMManager = VMManager::uninitialized();
// pub static USB: Usb = Usb::uninitialized();
// pub static GLOABAL_IRQ: GlobalIrq = GlobalIrq::new();
// pub static FIQ: Fiq = Fiq::new();
// pub static ETHERNET: GlobalEthernetDriver = GlobalEthernetDriver::uninitialized();

// extern "C" {
//     static __text_beg: u64;
//     static __text_end: u64;
//     static __bss_beg: u64;
//     static __bss_end: u64;
// }

// unsafe fn kmain() -> ! {
//     spin_sleep(Duration::from_secs(2));
//     crate::logger::init_logger();
//     info!(
//         "text beg: {:016x}, end: {:016x}",
//         &__text_beg as *const _ as u64, &__text_end as *const _ as u64
//     );
//     info!(
//         "bss  beg: {:016x}, end: {:016x}",
//         &__bss_beg as *const _ as u64, &__bss_end as *const _ as u64
//     );
//     ALLOCATOR.initialize();
//     FILESYSTEM.initialize();
//     VMM.initialize();
//     SCHEDULER.initialize();
//     init::initialize_app_cores();
//     VMM.wait();
//     kprintln!("starting processes...");
//     SCHEDULER.start();
//     loop {}
// }


