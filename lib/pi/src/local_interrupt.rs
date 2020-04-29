use core::time::Duration;
use shim::const_assert_size;
use aarch64::*;

use volatile::prelude::*;
use volatile::Volatile;

const INT_BASE: usize = 0x40000000;

/// Core interrupt sources (QA7: 4.10)
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LocalInterrupt {
    // Lab 5 1.C
    CNTPSIRQ = 0,
    CNTPNSIRQ = 1,
    CNTHPIRQ = 2,
    CNTVIRQ = 3,
    mailbox_0 = 4,
    mailbox_1 = 5,
    mailbox_2 = 6,
    mailbox_3 = 7,
    gpu = 8,
    pmu = 9,
    axi_outstanding_int = 10,
    local_timer_interrupt = 11
}

impl LocalInterrupt {
    pub const MAX: usize = 12;

    pub fn iter() -> impl Iterator<Item = LocalInterrupt> {
        (0..LocalInterrupt::MAX).map(|n| LocalInterrupt::from(n))
    }
}

impl From<usize> for LocalInterrupt {
    fn from(irq: usize) -> LocalInterrupt {
        // Lab 5 1.C
        use LocalInterrupt::*;
        match irq {
            0 => CNTPSIRQ,
            1 => CNTPNSIRQ,
            2 => CNTHPIRQ,
            3 => CNTVIRQ,
            4 => mailbox_0,
            5 => mailbox_1,
            6 => mailbox_2,
            7 => mailbox_3,
            8 => gpu,
            9 => pmu,
            10 => axi_outstanding_int,
            11 => local_timer_interrupt,
            _ => panic!("Unknown irq: {}", irq)
        }
    }
}

/// BCM2837 Local Peripheral Registers (QA7: Chapter 4)
#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    // Lab 5 1.C
    control: Volatile<u32>,
    _0: Volatile<u32>,
    core_timer_prescalar: Volatile<u32>,
    gpu_interrupts_routing: Volatile<u32>,
    perf_monitor_interrupts_set: Volatile<u32>,
    perf_monitor_interrupts_clr: Volatile<u32>,
    _1: Volatile<u32>,
    core_timer_ls_32_bits: Volatile<u32>,
    core_timer_ms_32_bits: Volatile<u32>,
    local_int_0: Volatile<u32>,
    _2: Volatile<u32>,
    axi_outstanding_counters: Volatile<u32>,
    axi_outstanding_irq: Volatile<u32>,
    local_timer_ctrl_status: Volatile<u32>,
    local_timer_write_flags: Volatile<u32>,
    _3: Volatile<u32>,
    core0_timers_interrupt_controller: Volatile<u32>,
    core1_timers_interrupt_controller: Volatile<u32>,
    core2_timers_interrupt_controller: Volatile<u32>,
    core3_timers_interrupt_controller: Volatile<u32>,
    core0_mailbox_interrupt_controller: Volatile<u32>,
    core1_mailbox_interrupt_controller: Volatile<u32>,
    core2_mailbox_interrupt_controller: Volatile<u32>,
    core3_mailbox_interrupt_controller: Volatile<u32>,
    core0_irq_src: Volatile<u32>,
    core1_irq_src: Volatile<u32>,
    core2_irq_src: Volatile<u32>,
    core3_irq_src: Volatile<u32>,
    core0_fiq_src: Volatile<u32>,
    core1_fiq_src: Volatile<u32>,
    core2_fiq_src: Volatile<u32>,
    core3_fiq_src: Volatile<u32>
}

const_assert_size!(Registers, 128);

pub struct LocalController {
    core: usize,
    registers: &'static mut Registers,
}

impl LocalController {
    /// Returns a new handle to the interrupt controller.
    pub fn new(core: usize) -> LocalController {
        LocalController {
            core: core,
            registers: unsafe { &mut *(INT_BASE as *mut Registers) },
        }
    }

    pub fn enable_local_timer(&mut self) {
        // Lab 5 1.C
        // Preserve the previos contents of the register
        unsafe {
            let prev = CNTP_CTL_EL0.get();
        // Set enable = 1
        let mut new = prev | 0b1;
        // Set IMASK = 0
        new = new & !0b10;
        CNTP_CTL_EL0.set(new);
        }

        //clear bit 5
        // enable CNTPS IRQ for specific core
        match self.core {
            0 => {
                self.registers.core0_timers_interrupt_controller.or_mask(0b10);
                self.registers.core0_timers_interrupt_controller.and_mask(0xDF);
            },
            1 => {
                self.registers.core1_timers_interrupt_controller.or_mask(0b10)

            },
            2 => self.registers.core2_timers_interrupt_controller.or_mask(0b10),
            3 => self.registers.core3_timers_interrupt_controller.or_mask(0b10),
            _ => panic!("ahhh when the cortex get so many cores")
        };
    }

    pub fn is_pending(&self, int: LocalInterrupt) -> bool {
        // Lab 5 1.C
        let index = int as usize;
        match self.core {
            0 => self.registers.core0_irq_src.has_mask(1 << index),
            1 => self.registers.core1_irq_src.has_mask(1 << index),
            2 => self.registers.core2_irq_src.has_mask(1 << index),
            3 => self.registers.core3_irq_src.has_mask(1 << index),
            _ => panic!("ahhh when the cortex get so many cores")
        }
    }

    pub fn tick_in(&mut self, t: Duration) {
        // Lab 5 1.C
        // See timer: 3.1 to 3.3
        unsafe {
            let hz = CNTFRQ_EL0.get();
            CNTP_TVAL_EL0.set(hz * t.as_secs() as u64);
        }
        // local_timer_write_flag write 31

    }
}

pub fn local_tick_in(core: usize, t: Duration) {
    LocalController::new(core).tick_in(t);
}
