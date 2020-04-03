use core::fmt;

#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
pub struct TrapFrame {
    // FIXME: Fill me in.
    TTBR0_EL1: u64,
    TTBR1_EL1: u64,
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
    pub fn set_spsr(&mut self, val: u64) {
        self.spsr = val;
    }
    pub fn get_sp(&self) -> u64 {
        self.sp
    }
    pub fn set_sp(&mut self, val: u64) {
        self.sp = val;
    }
    pub fn get_tpidr(&self) -> u64 {
        self.tpidr
    }
    pub fn set_tpidr(&mut self, val: u64) {
        self.tpidr = val;
    }
    pub fn get_gpr(&self, reg_num: u8) -> u64 {
        self.gprs[reg_num as usize]
    }
    pub fn set_gpr(&mut self, reg_num: u8, val: u64) {
        self.gprs[reg_num as usize] = val;
    }
    pub fn get_ttbr0(&self) -> u64 {
        self.TTBR0_EL1
    }
    pub fn set_ttbr0(&mut self, val: u64) {
        self.TTBR0_EL1 = val;
    }
    pub fn get_ttbr1(&self) -> u64 {
        self.TTBR1_EL1
    }
    pub fn set_ttbr1(&mut self, val: u64) {
        self.TTBR1_EL1 = val;
    }
    
    
}

