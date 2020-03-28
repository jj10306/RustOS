mod frame;
mod syndrome;
mod syscall;

pub mod irq;
pub use self::frame::TrapFrame;

use alloc::boxed::Box;
use pi::interrupt::{Controller, Interrupt};

use self::syndrome::Syndrome;
use self::syscall::handle_syscall;

use aarch64;
use crate::console::kprintln;
use crate::shell::shell;
use crate:: IRQ;
use crate::param::{TICK};

use pi::timer;

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Kind {
    Synchronous = 0,
    Irq = 1,
    Fiq = 2,
    SError = 3,
}

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Source {
    CurrentSpEl0 = 0,
    CurrentSpElx = 1,
    LowerAArch64 = 2,
    LowerAArch32 = 3,
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Info {
    source: Source,
    kind: Kind,
}

/// This function is called when an exception occurs. The `info` parameter
/// specifies the source and kind of exception that has occurred. The `esr` is
/// the value of the exception syndrome register. Finally, `tf` is a pointer to
/// the trap frame for the exception.
#[no_mangle]
pub extern "C" fn handle_exception(info: Info, esr: u32, tf: &mut TrapFrame) {
    //change this to tf.get_elr()
    

    //Note that you should only call Syndrome::from() for synchronous exceptions. 
    //The ESR_ELx register is not guaranteed to hold a valid value otherwise.
    // if info.kind == Info::Synchronous {
        kprintln!("info: {:?}", info);
        match info.kind {
            Kind::Synchronous => {
                tf.set_elr(tf.get_elr() + 4);
                let syndrome = Syndrome::from(esr);
                match syndrome {
                    Syndrome::Brk(comment) => {
                        shell("(dbg)$ ")
                    },
                    _ => {kprintln!("not Brk")}
                };
            },
            Kind::Irq => {
                kprintln!("In the IRQ branch");
                // let index = 0;
                // for interrupt in Interrupt::iter() {
                //     if Controller
                // }
                // IRQ.register(Interrupt::Timer1, Box::new(timer_handler));
                IRQ.invoke(Interrupt::Timer1, tf);
            },
            _ => { kprintln!("Not synchronous or Irq"); }
        }

}


