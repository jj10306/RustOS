use aarch64::ESR_EL1;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Fault {
    AddressSize,
    Translation,
    AccessFlag,
    Permission,
    Alignment,
    TlbConflict,
    Other(u8),
}

impl From<u32> for Fault {
    fn from(val: u32) -> Fault {
        use self::Fault::*;
        match val {
            0b000000 | 0b000001 | 0b000010 | 0b000011 => {
                AddressSize
            },
            0b000100 | 0b000101 | 0b000110 | 0b000111 => {
                Translation
            },
            0b001001 | 0b001010 | 0b001011 => {
                AccessFlag
            },
            0b001101 | 0b001110 | 0b001111 => {
                Permission
            },
            0b010000 | 0b011000 | 0b010100 | 0b010101 | 0b010110 | 0b010111 | 0b011100 | 0b011101 | 0b011110 | 0b011111 => {
                Alignment
            },
            0b110000 => {
                TlbConflict
            },
            _ => {
                Other(val as u8)
            }
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Syndrome {
    Unknown,
    WfiWfe,
    SimdFp,
    IllegalExecutionState,
    Svc(u16),
    Hvc(u16),
    Smc(u16),
    MsrMrsSystem,
    InstructionAbort { kind: Fault, level: u8 },
    PCAlignmentFault,
    DataAbort { kind: Fault, level: u8 },
    SpAlignmentFault,
    TrappedFpu,
    SError,
    Breakpoint,
    Step,
    Watchpoint,
    Brk(u16),
    Other(u32),
}

/// Converts a raw syndrome value (ESR) into a `Syndrome` (ref: D1.10.4).
impl From<u32> for Syndrome {
    fn from(esr: u32) -> Syndrome {
        use self::Syndrome::*;
        match ESR_EL1::get_value(esr as u64, ESR_EL1::EC) {
            0b000000 => {
                Unknown
            },
            0b000001 => {
                WfiWfe
            },
            0b000111 => {
                SimdFp
            },
            0b001110 => {
                IllegalExecutionState
            },
            0b010101 => {
                //Svc
                let imm16 = ESR_EL1::get_value(esr as u64, ESR_EL1::ISS_HSVC_IMM) as u16;
                Svc(imm16)
            },
            0b010110 => {
                //Hvc
                let imm16 = ESR_EL1::get_value(esr as u64, ESR_EL1::ISS_HSVC_IMM) as u16;
                Hvc(imm16)
            },
            0b010111 => {
                //Smc
                let imm16 = ESR_EL1::get_value(esr as u64, ESR_EL1::ISS_HSVC_IMM) as u16;
                Smc(imm16)
            },
            0b011000 => {
                MsrMrsSystem
            },
            0b100000 | 0b100001 => {
                //InstructionAbort
                let fault_bits = ESR_EL1::get_value(esr as u64, 0b111_111);
                let kind = Fault::from(fault_bits as u32);
                let level = ESR_EL1::get_value(esr as u64, 0b11) as u8;
                InstructionAbort { kind, level }
            },
            0b100010 => {
                PCAlignmentFault
            },
            0b100100 | 0b100101 => {
                //DataAbort
                let fault_bits = ESR_EL1::get_value(esr as u64, 0b111_111);
                let kind = Fault::from(fault_bits as u32);
                let level = ESR_EL1::get_value(esr as u64, 0b11) as u8;
                InstructionAbort { kind, level }
            },
            0b100110 => {
                SpAlignmentFault
            },
            0b101100 => {
                TrappedFpu
            },
            0b101111 => {
                SError
            },
            0b110000 | 0b110001 => {
                Breakpoint
            },
            0b110010 | 0b110011 => {
                Step
            },
            0b110100 | 0b110101 => {
                Watchpoint
            },
            0b111100 => {
                let comment = ESR_EL1::get_value(esr as u64, ESR_EL1::ISS_BRK_CMMT) as u16;
                Brk(comment)
            },
            _ => {
                Other(esr)
            }
        }
    }
}
