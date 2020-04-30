use aarch64::*;
use pi::common::SPINNING_BASE;
use core::mem::zeroed;
use core::ptr::{write_volatile, read_volatile};
use crate::kprintln;
use crate::SCHEDULER;

mod oom;
mod panic;

use crate::kmain;
use crate::param::*;
use crate::VMM;

global_asm!(include_str!("init/vectors.s"));

//
// big assumptions (better to be checked):
//   _start1/2(), _kinit1/2(), switch_to_el1/2() should NOT use stack!
//   e.g., #[no_stack] would be useful ..
//
// so, no debug build support!
//

/// Kernel entrypoint for core 0
#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    if MPIDR_EL1.get_value(MPIDR_EL1::Aff0) == 0 {
        SP.set(KERN_STACK_BASE);
        kinit()
    }
    unreachable!()
}

unsafe fn zeros_bss() {
    extern "C" {
        static mut __bss_beg: u64;
        static mut __bss_end: u64;
    }

    let mut iter: *mut u64 = &mut __bss_beg;
    let end: *mut u64 = &mut __bss_end;

    while iter < end {
        write_volatile(iter, zeroed());
        iter = iter.add(1);
    }
}

#[no_mangle]
unsafe fn switch_to_el2() {
    if current_el() == 3 {
        // set up Secure Configuration Register (D13.2.10)
        SCR_EL3.set(SCR_EL3::NS | SCR_EL3::SMD | SCR_EL3::HCE | SCR_EL3::RW | SCR_EL3::RES1);

        // set up Saved Program Status Register (C5.2.19)
        SPSR_EL3
            .set((SPSR_EL3::M & 0b1001) | SPSR_EL3::F | SPSR_EL3::I | SPSR_EL3::A | SPSR_EL3::D);

        // eret to itself, expecting current_el() == 2 this time.
        ELR_EL3.set(switch_to_el2 as u64);
        asm::eret();
    }
}

#[no_mangle]
unsafe fn switch_to_el1() {
    extern "C" {
        static mut vectors: u64;
    }

    if current_el() == 2 {
        // set the stack-pointer for EL1
        SP_EL1.set(SP.get() as u64);

        // enable CNTP for EL1/EL0 (ref: D7.5.2, D7.5.13)
        // NOTE: This doesn't actually enable the counter stream.
        CNTHCTL_EL2.set(CNTHCTL_EL2.get() | CNTHCTL_EL2::EL0VCTEN | CNTHCTL_EL2::EL0PCTEN);
        CNTVOFF_EL2.set(0);


        // enable AArch64 in EL1 (A53: 4.3.36)
        HCR_EL2.set(HCR_EL2::RW | HCR_EL2::RES1);
        // enable floating point and SVE (SIMD) (A53: 4.3.38, 4.3.34)
        CPTR_EL2.set(0);
        CPACR_EL1.set(CPACR_EL1.get() | (0b11 << 20));

        // Set SCTLR to known state (A53: 4.3.30)
        SCTLR_EL1.set(SCTLR_EL1::RES1);

        // set up exception handlers
        // FIXME: load `vectors` addr into appropriate register (guide: 10.4)

        VBAR_EL1.set(&vectors as *const u64 as u64);
        // change execution level to EL1 (ref: C5.2.19)
        SPSR_EL2.set(
            (SPSR_EL2::M & 0b0101) // EL1h
            | SPSR_EL2::F
            | SPSR_EL2::I
            | SPSR_EL2::D
            | SPSR_EL2::A,
        );

        // FIXME: eret to itself, expecting current_el() == 1 this time
        ELR_EL2.set(switch_to_el1 as u64);
        asm::eret();
    }
}

#[no_mangle]
unsafe fn kinit() -> ! {
    zeros_bss();
    switch_to_el2();
    switch_to_el1();
    kmain();
}

/// Kernel entrypoint for core 1, 2, and 3
#[no_mangle]
pub unsafe extern "C" fn start2() -> ! {
    // Lab 5 1.A
    SP.set(KERN_STACK_BASE - KERN_STACK_SIZE * MPIDR_EL1.get_value(MPIDR_EL1::Aff0) as usize);
    kinit2()
}

unsafe fn kinit2() -> ! {
    switch_to_el2();
    switch_to_el1();
    kmain2()
}

unsafe fn kmain2() -> ! {
    // Lab 5 1.A
    // Notify core 0 that this core is fully awake
    write_volatile(SPINNING_BASE.add(affinity()), zeroed());
    VMM.wait();
    let n = affinity();
    SCHEDULER.start()
    // loop {
    // '    // match n {
    //     //     1 => kprintln!("11111111"),
    //     //     2 => kprintln!("22222222"),
    //     //     3 => kprintln!("33333333"),
    //     //     _ => panic!("oops")
    //     // }
        
    // }
}

/// Wakes up each app core by writing the address of `init::start2`
/// to their spinning base and send event with `sev()`.
pub unsafe fn initialize_app_cores() {
    // Lab 5 1.A
    let start2_addr = start2 as usize;
    for i in (1..=NCORES - 1) {
        let curr_core_spinning_addr = SPINNING_BASE.add(i);
        write_volatile(curr_core_spinning_addr, start2_addr);
        sev();
        while read_volatile(curr_core_spinning_addr) != 0 {
            //wait until Core 0 receives acknowledgment from other core
        }
    }
}