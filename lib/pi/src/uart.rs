use core::fmt;
use core::time::Duration;

use shim::io;
use shim::const_assert_size;

use volatile::prelude::*;
use volatile::{Volatile, ReadVolatile, Reserved};

use crate::timer;
use crate::common::IO_BASE;
use crate::gpio::{Gpio, Function};

/// The base address for the `MU` registers.
const MU_REG_BASE: usize = IO_BASE + 0x215040;

/// The `AUXENB` register from page 9 of the BCM2837 documentation.
const AUX_ENABLES: *mut Volatile<u8> = (IO_BASE + 0x215004) as *mut Volatile<u8>;

/// Enum representing bit fields of the `AUX_MU_LSR_REG` register.
/// The AUX_MU_LSR_REG register shows the data status.  
#[repr(u8)]
enum LsrStatus {
    DataReady = 1,
    TxAvailable = 1 << 5,
}

#[repr(C)]
#[allow(non_snake_case)]

struct Registers {
    // FIXME: Declare the "MU" registers from page 8.

    /// The AUX_MU_IO_REG register is primary used to write data to and read data from the
    /// UART FIFOs. 
    IO: Volatile<u8>,
    /// The AUX_MU_IER_REG register is primary used to enable interrupts 
    IER: Volatile<u8>,
    /// The AUX_MU_IIR_REG register shows the interrupt status. 
    IIR: Volatile<u8>,
    /// The AUX_MU_LCR_REG register controls the line data format and gives access to the
    /// baudrate register 
    LCR: Volatile<u8>,
    /// The AUX_MU_MCR_REG register controls the 'modem' signals. 
    MCR: Volatile<u8>,
    /// The AUX_MU_LSR_REG register shows the data status. 
    LSR: Volatile<u8>,
    /// The AUX_MU_MSR_REG register shows the 'modem' status. 
    MSR: ReadVolatile<u8>,
    /// The AUX_MU_SCRATCH is a single byte storage. 
    SCRATCH:  Volatile<u8>,
    /// The AUX_MU_CNTL_REG provides access to some extra useful and nice features not
    /// found on a normal 16550 UART .
    CNTL: Volatile<u8>,
    /// The AUX_MU_STAT_REG provides a lot of useful information about the internal status of
    /// the mini UART not found on a normal 16550 UART. 
    STAT: ReadVolatile<u32>,
    /// The AUX_MU_BAUD register allows direct access to the 16-bit wide baudrate counter. 
    BAUD: Volatile<u32>

}

/// The Raspberry Pi's "mini UART".
pub struct MiniUart {
    registers: &'static mut Registers,
    timeout: Option<Duration>,
}

impl MiniUart {
    /// Initializes the mini UART by enabling it as an auxiliary peripheral,
    /// setting the data size to 8 bits, setting the BAUD rate to ~115200 (baud
    /// divider of 270), setting GPIO pins 14 and 15 to alternative function 5
    /// (TXD1/RDXD1), and finally enabling the UART transmitter and receiver.
    ///
    /// By default, reads will never time out. To set a read timeout, use
    /// `set_read_timeout()`.
    pub fn new() -> MiniUart {
        let registers = unsafe {
            // Enable the mini UART as an auxiliary device.
            (*AUX_ENABLES).or_mask(1);
            &mut *(MU_REG_BASE as *mut Registers) 

        };

            //setting the data size to 8 bits
            registers.LCR.or_mask(0b11);

            //setting the BAUD rate to ~115200 (baud divider of 270)
            registers.BAUD.and_mask(0);
            registers.BAUD.or_mask(0b11100001000000000);

            //setting GPIO pins 14 and 15 to alternative function 5 (TXD1/RDXD1)
            Gpio::new(14).into_alt(Function::Alt5);
            Gpio::new(15).into_alt(Function::Alt5);

            //enabling the UART transmitter and receiver
            registers.CNTL.or_mask(0b11);
            
            MiniUart {
                registers,
                timeout: None
            }
    }

    /// Set the read timeout to `t` duration.
    pub fn set_read_timeout(&mut self, t: Duration) {
        self.timeout = Some(t);
    }

    /// Write the byte `byte`. This method blocks until there is space available
    /// in the output FIFO.
    pub fn write_byte(&mut self, byte: u8) {
        unimplemented!()
    }

    /// Returns `true` if there is at least one byte ready to be read. If this
    /// method returns `true`, a subsequent call to `read_byte` is guaranteed to
    /// return immediately. This method does not block.
    pub fn has_byte(&self) -> bool {
        unimplemented!()
    }

    /// Blocks until there is a byte ready to read. If a read timeout is set,
    /// this method blocks for at most that amount of time. Otherwise, this
    /// method blocks indefinitely until there is a byte to read.
    ///
    /// Returns `Ok(())` if a byte is ready to read. Returns `Err(())` if the
    /// timeout expired while waiting for a byte to be ready. If this method
    /// returns `Ok(())`, a subsequent call to `read_byte` is guaranteed to
    /// return immediately.
    pub fn wait_for_byte(&self) -> Result<(), ()> {
        unimplemented!()
    }

    /// Reads a byte. Blocks indefinitely until a byte is ready to be read.
    pub fn read_byte(&mut self) -> u8 {
        unimplemented!()
    }
}

// FIXME: Implement `fmt::Write` for `MiniUart`. A b'\r' byte should be written
// before writing any b'\n' byte.

mod uart_io {
    use super::io;
    use super::MiniUart;
    use volatile::prelude::*;

    // FIXME: Implement `io::Read` and `io::Write` for `MiniUart`.
    //
    // The `io::Read::read()` implementation must respect the read timeout by
    // waiting at most that time for the _first byte_. It should not wait for
    // any additional bytes but _should_ read as many bytes as possible. If the
    // read times out, an error of kind `TimedOut` should be returned.
    //
    // The `io::Write::write()` method must write all of the requested bytes
    // before returning.
}
