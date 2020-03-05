////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

use core::alloc::Layout;
use core::fmt;
use core::ptr;

use crate::allocator::linked_list::LinkedList;
use crate::allocator::util::*;
use crate::allocator::LocalAlloc;

// maps a requested allocation's size to a bin #
// returns 30 if requested size is too large or if overflow occurs
pub fn map_size_to_bin(size: usize) -> usize {
    // consider checking if size < 8
    if size <= 8 {
        return 0; 
    } else {
        let next_power = match size.checked_next_power_of_two() {
            Some(pow) => pow,
            None => return 30
        };
        let mut exp = 0;
        let mut acc = 1;
        while acc != next_power {
            acc *= 2;
            exp += 1;    
        }
        // if value is 2^x, the bin is x - 3
        exp - 3
        
    }
}
/// A simple allocator that allocates based on size classes.
///   bin 0 (2^3 bytes)    : handles allocations in (0, 2^3]
///   bin 1 (2^4 bytes)    : handles allocations in (2^3, 2^4]
///   ...
///   bin 29 (2^22 bytes): handles allocations in (2^31, 2^32]
///   
///   map_to_bin(size) -> k
pub struct Allocator {
    // FIXME: Add the necessary fields.

    // these two fields mimic the Bump allocator
    gloabl_pool_current: usize,
    gloabl_pool_end: usize,
    // heads of the linked lists that contain all the blocks of that particular size
    bins: [LinkedList; 30]

}

impl Allocator {
    /// Creates a new bin allocator that will allocate memory from the region
    /// starting at address `start` and ending at address `end`.
    pub fn new(start: usize, end: usize) -> Allocator {
        Allocator {
            gloabl_pool_current: start,
            gloabl_pool_end: end,
            bins: [LinkedList::new(); 30]
        }
    }

    // invoked when no free blocks are available, thus it is necessary to 'degenerate' to 
    // bump-like allocation
    fn bump(&mut self, layout: Layout) -> Option<*mut u8> {
        // guranteed that alignment is power of two
        let alloc_start = align_up(self.gloabl_pool_current, layout.align());
        let power_of_two_size = layout.size();

        let alloc_end = match alloc_start.checked_add(power_of_two_size) {
            Some(end) => end,
            None => return None
        };
        if alloc_end > self.gloabl_pool_end {
            None
        } else {
            self.gloabl_pool_current = alloc_end;
            Some(alloc_start as *mut u8)
        }
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
        let bin_num = map_size_to_bin(layout.size());

        if bin_num > 29 || !layout.align().is_power_of_two() {
            core::ptr::null_mut()
        } else {
            // the linked list

            for i in bin_num .. 30 {
                let mut ll = self.bins[i];
                // where is the Node {    } struct stored at? the stack?
                for node in ll.iter_mut() {
                    if node.value() as usize % layout.align() == 0 {
                        return node.pop() as *mut u8;
                    }
                }
            }   
            // improved implementation checks larger bins for block and then redistributes remaining memory
            
            // naive implementation simply bumps if there is no block with exact same size
            match self.bump(layout) {
                Some(ptr) => ptr,
                None => core::ptr::null_mut()
            }
        
        }   
            // round the allocation size to the nearest power 
            // let power_of_two_size = layout.size().next_power_of_two();
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
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        let bin_num = map_size_to_bin(layout.size());
        // let aligned_ptr = align_up(ptr, layout.align());
        self.bins[bin_num].push(ptr as *mut usize);
    }
}

// FIXME: Implement `Debug` for `Allocator`.
impl fmt::Debug for Allocator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Global Pool Current: {}, Global Pool End: {}, Bins: {:?}", self.gloabl_pool_current, self.gloabl_pool_end, self.bins)

    }
}
