use core::fmt;

#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
pub struct TrapFrame {
    // FIXME: Fill me in.
    elr: u64,
    spsr: u64,
    sp: u64,
    tpidr: u64,
    qs: [u128; 32],
    gprs: [u64; 32],
}
impl TrapFrame {
    pub fn get_elr(&self) -> u64 {
        self.elr
    }
    pub fn set_elr(&mut self, val: u64) {
        self.elr = val;
    }
    pub fn get_spsr(&self) -> u64 {
        self.spsr
    }
    pub fn get_sp(&self) -> u64 {
        self.sp
    }
    pub fn get_tpidr(&self) -> u64 {
        self.tpidr
    }
}

