use core::alloc::Layout;
use core::ptr;

use crate::allocator::util::*;
use crate::allocator::{ LocalAlloc, memory_map };

/// A "bump" allocator: allocates memory by bumping a pointer; never frees.
#[derive(Debug)]
pub struct Allocator {
    current: usize,
    end: usize,
}

impl Allocator {
    /// Creates a new bump allocator that will allocate memory from the region
    /// starting at address `start` and ending at address `end`.
    #[allow(dead_code)]
    pub fn new(start: usize, end: usize) -> Allocator {
        if end > start {
            return Allocator { current: start, end };
        }
        panic!("End address must be greater than start address to initialize an allocator");
        
    }
}

impl LocalAlloc for Allocator {
    /// Allocates memory. Returns a pointer meeting the size and alignment
    /// properties of `layout.size()` and `layout.align()`.
    ///
    /// If this method returns an `Ok(addr)`, `addr` will be non-null address
    /// pointing to a block of storage suitable for holding an instance of
    /// `layout`. In particular, the block will be at least `layout.size()`
    /// bytes large and will be aligned to `layout.align()`. The returned block
    /// of storage may or may not have its contents initialized or zeroed.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure that `layout.size() > 0` and that
    /// `layout.align()` is a power of two. Parameters not meeting these
    /// conditions may result in undefined behavior.
    ///
    /// # Errors
    ///
    /// Returning null pointer (`core::ptr::null_mut`)
    /// indicates that either memory is exhausted
    /// or `layout` does not meet this allocator's
    /// size or alignment constraints.
    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        if !layout.align().is_power_of_two(){ return core::ptr::null_mut() }

        let rtn = align_up(self.current, layout.align());
        let this_end = match rtn.checked_add(layout.size()) {
            Some(end) => end,
            None => return core::ptr::null_mut()
        };
        if this_end > self.end {
            core::ptr::null_mut()
        } else {
            self.current = this_end;
            rtn as *mut u8
        }
    }

    /// Deallocates the memory referenced by `ptr`.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure the following:
    ///
    ///   * `ptr` must denote a block of memory currently allocated via this
    ///     allocator
    ///   * `layout` must properly represent the original layout used in the
    ///     allocation call that returned `ptr`
    ///
    /// Parameters not meeting these conditions may result in undefined
    /// behavior.
    unsafe fn dealloc(&mut self, _ptr: *mut u8, _layout: Layout) {
        //LEAK
    }
}