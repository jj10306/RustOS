use core::alloc::Layout;
use core::fmt;
use core::ptr;

use crate::allocator::linked_list::LinkedList;
use crate::allocator::util::*;
use crate::allocator::LocalAlloc;

// maps a requested allocation's size to a bin #
// returns 30 if requested size is too large or if overflow occurs
// pub fn map_size_to_bin(size: usize) -> usize {
//     // consider checking if size < 8
//     if size <= 8 {
//         return 0; 
//     } else {
//         let next_power = match size.checked_next_power_of_two() {
//             Some(pow) => pow,
//             None => return 30
//         };
//         let mut exp = 0;
//         let mut acc = 1;
//         while acc != next_power {
//             acc *= 2;
//             exp += 1;    
//         }
//         // if value is 2^x, the bin is x - 3
//         exp - 3
        
//     }
//}
// above implementation was flawed and inefficient, see new cleaner one below
fn map_size_to_bin(size: usize) -> usize {
    for lefty in 3..=32 {
        // upper first time the term gets dominated
        if size <= (1 << lefty) { 
            return lefty - 3; 
        }
    }
    return 0;
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
    bins: [LinkedList; 30],
    MAX: usize
}

impl Allocator {
    /// Creates a new bin allocator that will allocate memory from the region
    /// starting at address `start` and ending at address `end`.
    pub fn new(start: usize, end: usize) -> Allocator {
        Allocator {
            gloabl_pool_current: start,
            gloabl_pool_end: end,
            bins: [LinkedList::new(); 30],
            MAX: end - start
        }
    }

    //splits memory from the big bin all the way to the start bin
    fn split_memory(&mut self, mut address: usize, mut start_bin: usize, big_bin: usize) {
        while start_bin < big_bin {
            unsafe { self.bins[start_bin].push(address as *mut usize) }
            address += 1 << (start_bin + 3); //Leave space for the block
            start_bin += 1;
        }
    }

    // invoked when no free blocks are available, thus it is necessary to 'degenerate' to 
    // bump-like allocation
    fn bump(&mut self, size: usize, align: usize) -> Option<*mut u8> {
        // guranteed that alignment is power of two
        let addr = align_up(self.gloabl_pool_current, align );
        // saturation is necessary so we dont exceed 
        let new_start = addr.saturating_add(size);
        // in the extreme case that we run  out of mem
        if new_start > self.gloabl_pool_end {
            None
        } else {
            self.gloabl_pool_current = new_start;
            Some(addr as *mut u8)
        }
        // let alloc_start = align_up(self.gloabl_pool_current, layout.align());
        // let power_of_two_size = layout.size();

        // let alloc_end = match alloc_start.checked_add(power_of_two_size) {
        //     Some(end) => end,
        //     None => return None
        // };
        // if alloc_end > self.gloabl_pool_end {
        //     None
        // } else {
        //     self.gloabl_pool_current = alloc_end;
        //     Some(alloc_start as *mut u8)
        // }
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
        let bin_size = 1 << (bin_num + 3);
        if layout.size() <= 0 || !(layout.align().is_power_of_two()) || layout.size() > self.MAX {
            return core::ptr::null_mut();
        } else {
            // the linked list
            for i in bin_num .. 30 {
                // let mut ll = self.bins[i];
                // where is the Node {    } struct stored at? the stack?
                for node in self.bins[i].iter_mut() {
                    if node.value() as usize % layout.align() == 0 {
                        if i != bin_num{
                            self.split_memory(node.value() as usize + bin_size, bin_num, i);
                        }
                        return node.pop() as *mut u8;
                    }
                }
            }   
            // improved implementation checks larger bins for block and then redistributes remaining memory
            
            // naive implementation simply bumps if there is no block with exact same size
            match self.bump(bin_size, layout.align()) {
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
