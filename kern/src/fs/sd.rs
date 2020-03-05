use core::time::Duration;
use shim::io;
use shim::ioerr;

use pi::timer::spin_sleep;
use fat32::traits::BlockDevice;

extern "C" {
    /// A global representing the last SD controller error that occured.
    static sd_err: i64;

    /// Initializes the SD card controller.
    ///
    /// Returns 0 if initialization is successful. If initialization fails,
    /// returns -1 if a timeout occured, or -2 if an error sending commands to
    /// the SD controller occured.
    fn sd_init() -> i32;

    /// Reads sector `n` (512 bytes) from the SD card and writes it to `buffer`.
    /// It is undefined behavior if `buffer` does not point to at least 512
    /// bytes of memory. Also, the caller of this function should make sure that
    /// `buffer` is at least 4-byte aligned.
    ///
    /// On success, returns the number of bytes read: a positive number.
    ///
    /// On error, returns 0. The true error code is stored in the `sd_err`
    /// global. `sd_err` will be set to -1 if a timeout occured or -2 if an
    /// error sending commands to the SD controller occured. Other error codes
    /// are also possible but defined only as being less than zero.
    fn sd_readsector(n: i32, buffer: *mut u8) -> i32;
}

// FIXME: Define a `#[no_mangle]` `wait_micros` function for use by `libsd`.
// The `wait_micros` C signature is: `void wait_micros(unsigned int);`
#[no_mangle]
fn wait_micros(micros: u32) {
    spin_sleep(core::time::Duration::from_micros(micros as u64));
}

/// A handle to an SD card controller.
#[derive(Debug)]
pub struct Sd;

impl Sd {
    /// Initializes the SD card controller and returns a handle to it.
    /// The caller should assure that the method is invoked only once during the
    /// kernel initialization. We can enforce the requirement in safe Rust code
    /// with atomic memory access, but we can't use it yet since we haven't
    /// written the memory management unit (MMU).
    pub unsafe fn new() -> Result<Sd, io::Error> {
        let code;
        unsafe {
            code = sd_init();
        }

        if code == 0 {
            return Ok(Sd);
        } else if code == -1 {
            return ioerr!(TimedOut, "SD Timeout Occurred");
        } else if code == -2 {
            return ioerr!(Other, "Error sending commands to the SD Card");
        } else {
            return ioerr!(Other, "Some weird ish occurring");
        }
    }
    }

impl BlockDevice for Sd {
    /// Reads sector `n` from the SD card into `buf`. On success, the number of
    /// bytes read is returned.
    ///
    /// # Errors
    ///
    /// An I/O error of kind `InvalidInput` is returned if `buf.len() < 512` or
    /// `n > 2^31 - 1` (the maximum value for an `i32`).
    ///
    /// An error of kind `TimedOut` is returned if a timeout occurs while
    /// reading from the SD card.
    ///
    /// An error of kind `Other` is returned for all other errors.
    fn read_sector(&mut self, n: u64, buf: &mut [u8]) -> io::Result<usize> {
        let two = 2u64;
        if buf.len() < 512 || n > two.pow(31) - 1 {
            return ioerr!(InvalidInput, "Invalid Input");
        } 
        let num;
        unsafe {
            num = sd_readsector(n as i32, &mut buf[0] as *mut u8);
        }
        if num == 0 {
            unsafe {
                match sd_err {
                    -1 => {return ioerr!(TimedOut, "Timeout occurred");},
                    -2 => {return ioerr!(Other, "Something weird occurring ah");}
                    _ => {return ioerr!(Other, "why u here");}
                }
            }
        }
        // panic!("{:?}", buf);
        Ok(num as usize)
    }

    fn write_sector(&mut self, _n: u64, _buf: &[u8]) -> io::Result<usize> {
        unimplemented!("SD card and file system are read only")
    }
}
