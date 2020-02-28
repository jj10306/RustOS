use core::marker::PhantomData;

use crate::common::{states, GPIO_BASE};
use volatile::prelude::*;
use volatile::{ReadVolatile, Reserved, Volatile, WriteVolatile};

/// An alternative GPIO function.
#[repr(u8)]
pub enum Function {
    Input = 0b000,
    Output = 0b001,
    Alt0 = 0b100,
    Alt1 = 0b101,
    Alt2 = 0b110,
    Alt3 = 0b111,
    Alt4 = 0b011,
    Alt5 = 0b010,
}

#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    FSEL: [Volatile<u32>; 6],
    __r0: Reserved<u32>,
    SET: [WriteVolatile<u32>; 2],
    __r1: Reserved<u32>,
    CLR: [WriteVolatile<u32>; 2],
    __r2: Reserved<u32>,
    LEV: [ReadVolatile<u32>; 2],
    __r3: Reserved<u32>,
    EDS: [Volatile<u32>; 2],
    __r4: Reserved<u32>,
    REN: [Volatile<u32>; 2],
    __r5: Reserved<u32>,
    FEN: [Volatile<u32>; 2],
    __r6: Reserved<u32>,
    HEN: [Volatile<u32>; 2],
    __r7: Reserved<u32>,
    LEN: [Volatile<u32>; 2],
    __r8: Reserved<u32>,
    AREN: [Volatile<u32>; 2],
    __r9: Reserved<u32>,
    AFEN: [Volatile<u32>; 2],
    __r10: Reserved<u32>,
    PUD: Volatile<u32>,
    PUDCLK: [Volatile<u32>; 2],
}

/// Possible states for a GPIO pin.
#[allow(unused_doc_comments)]
states! {
    Uninitialized, Input, Output, Alt
}

/// A GPIO pin in state `State`.
///
/// The `State` generic always corresponds to an uninstantiatable type that is
/// use solely to mark and track the state of a given GPIO pin. A `Gpio`
/// structure starts in the `Uninitialized` state and must be transitions into
/// one of `Input`, `Output`, or `Alt` via the `into_input`, `into_output`, and
/// `into_alt` methods before it can be used.
pub struct Gpio<State> {
    pin: u8,
    registers: &'static mut Registers,
    _state: PhantomData<State>,
}

/// The base address of the `GPIO` registers.
//const GPIO_BASE: usize = IO_BASE + 0x200000; //why does the IO_BASE start at 0x3F000000

impl<T> Gpio<T> {
    /// Transitions `self` to state `S`, consuming `self` and returning a new
    /// `Gpio` instance in state `S`. This method should _never_ be exposed to
    /// the public!
    #[inline(always)]
    fn transition<S>(self) -> Gpio<S> {
        Gpio {
            pin: self.pin,
            registers: self.registers,
            _state: PhantomData,
        }
    }
}

impl Gpio<Uninitialized> {
    /// Returns a new `GPIO` structure for pin number `pin`.
    ///
    /// # Panics
    ///
    /// Panics if `pin` > `53`.
    pub fn new(pin: u8) -> Gpio<Uninitialized> {
        if pin > 53 {
            panic!("Gpio::new(): pin {} exceeds maximum of 53", pin);
        }

        Gpio {
            registers: unsafe { &mut *(GPIO_BASE as *mut Registers) },
            pin: pin,
            _state: PhantomData,
        }
    }

    /// Enables the alternative function `function` for `self`. Consumes self
    /// and returns a `Gpio` structure in the `Alt` state.
    pub fn into_alt(self, function: Function) -> Gpio<Alt> {
        //why do you want to go from Alt -> X, I thought Alt had no transitions

        //there are 10 pins per register with 4 in the last register, thus there are 6 reg (0 - 5)
        let func_sel_num = (self.pin / 10) as usize; 
        //3 bits per 10 pins per reg 
        let func_sel_reg_offset = ((self.pin % 10) * 3) as usize;

        //why does this need to be a reference specifically mutable
        //why does the variable itself not have to be mutable
        // grabbes the correct function select register
        let func_sel_reg: &mut Volatile<u32> = &mut self.registers.FSEL[func_sel_num];
        //mask the 3 bits in the place we want to mutate with 0s
        func_sel_reg.and_mask(! (0b111 << func_sel_reg_offset));
        //or mask the 3 bits we want to set the to the appropriate value
        func_sel_reg.or_mask((function as u32) << func_sel_reg_offset);

        Gpio {
            registers: self.registers,
            pin: self.pin,
            _state: PhantomData
        }
    }

    /// Sets this pin to be an _output_ pin. Consumes self and returns a `Gpio`
    /// structure in the `Output` state.
    pub fn into_output(self) -> Gpio<Output> {
        self.into_alt(Function::Output).transition()
    }

    /// Sets this pin to be an _input_ pin. Consumes self and returns a `Gpio`
    /// structure in the `Input` state.
    pub fn into_input(self) -> Gpio<Input> {
        self.into_alt(Function::Input).transition()
    }
}

impl Gpio<Output> {
    /// Sets (turns on) the pin.
    pub fn set(&mut self) {
        let pin_number = self.pin;
        //0 or 1
        let set_reg_num = (pin_number / 32) as usize;
        let set_reg_offset = pin_number % 32;
        let set_reg: &mut WriteVolatile<u32> = &mut self.registers.SET[set_reg_num];
        set_reg.write(0b1 << set_reg_offset);
    }

    /// Clears (turns off) the pin.
    pub fn clear(&mut self) {
        let pin_number = self.pin;
        //0 or 1
        let clear_reg_num = (pin_number / 32) as usize;
        let clear_reg_offset = pin_number % 32;
        let clear_reg: &mut WriteVolatile<u32> = &mut self.registers.CLR[clear_reg_num];
        clear_reg.write(0b1 << clear_reg_offset);
    }
}

impl Gpio<Input> {
    /// Reads the pin's value. Returns `true` if the level is high and `false`
    /// if the level is low.
    pub fn level(&mut self) -> bool {
        let pin_number = self.pin;
        let level_reg_num = (pin_number / 32) as usize;
        let level_reg_offset = pin_number % 32;
        let level_reg: &mut ReadVolatile<u32> = &mut self.registers.LEV[level_reg_num];
        level_reg.has_mask(0b1 << level_reg_offset)
        
    }
}
