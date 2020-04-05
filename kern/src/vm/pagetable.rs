
use core::iter::Chain;
use core::ops::{Deref, DerefMut};
use core::slice::Iter;

use alloc::boxed::Box;
use alloc::fmt;
use core::alloc::{GlobalAlloc, Layout};

use crate::allocator;
use crate::param::*;
use crate::vm::{PhysicalAddr, VirtualAddr};
use crate::ALLOCATOR;
use crate::console::kprintln;

use aarch64::vmsa::*;
use shim::const_assert_size;

#[repr(C)]
pub struct Page([u8; PAGE_SIZE]);
const_assert_size!(Page, PAGE_SIZE);

impl Page {
    pub const SIZE: usize = PAGE_SIZE;
    pub const ALIGN: usize = PAGE_SIZE;

    fn layout() -> Layout {
        unsafe { Layout::from_size_align_unchecked(Self::SIZE, Self::ALIGN) }
    }
}

#[repr(C)]
#[repr(align(65536))]
pub struct L2PageTable {
    pub entries: [RawL2Entry; 8192],
}
const_assert_size!(L2PageTable, PAGE_SIZE);

impl L2PageTable {
    /// Returns a new `L2PageTable`
    fn new() -> L2PageTable {
        let entries = [RawL2Entry::new(0); 8192];
        L2PageTable {
            entries
        }
    }

    /// Returns a `PhysicalAddr` of the pagetable.
    pub fn as_ptr(&self) -> PhysicalAddr {
        let addr = self as *const L2PageTable as *const usize as usize;
        let rtn: PhysicalAddr = From::from(addr);
        rtn
    }
}

#[derive(Copy, Clone)]
pub struct L3Entry(RawL3Entry);

impl L3Entry {
    /// Returns a new `L3Entry`.
    fn new() -> L3Entry {
        let raw_entry = RawL3Entry::new(0);
        L3Entry(raw_entry)
    }

    /// Returns `true` if the L3Entry is valid and `false` otherwise.
    fn is_valid(&self) -> bool {
        let raw_entry = self.0;
        raw_entry.get_value(RawL3Entry::VALID) == 0b1
    }

    /// Extracts `ADDR` field of the L3Entry and returns as a `PhysicalAddr`
    /// if valid. Otherwise, return `None`.
    fn get_page_addr(&self) -> Option<PhysicalAddr> {
        //what does it mean by "if valid"?
        let raw_entry = self.0;
        let addr: PhysicalAddr = From::from(raw_entry.get_value(RawL3Entry::ADDR));
        match self.is_valid() {
            true => Some(addr),
            false => None
        }
    }
}

#[repr(C)]
#[repr(align(65536))]
pub struct L3PageTable {
    pub entries: [L3Entry; 8192],
}
const_assert_size!(L3PageTable, PAGE_SIZE);

impl L3PageTable {
    /// Returns a new `L3PageTable`.
    fn new() -> L3PageTable {
        let entries = [L3Entry::new(); 8192];
        L3PageTable {
            entries
        }
    }

    /// Returns a `PhysicalAddr` of the pagetable.
    pub fn as_ptr(&self) -> PhysicalAddr {
        let addr = self as *const L3PageTable as *const usize as usize;
        let rtn: PhysicalAddr = From::from(addr);
        rtn
    }
}


#[repr(C)]
#[repr(align(65536))]
pub struct PageTable {
    pub l2: L2PageTable,
    pub l3: [L3PageTable; 2],
}

impl PageTable {
    /// Returns a new `Box` containing `PageTable`.
    /// Entries in L2PageTable should be initialized properly before return.
    fn new(perm: u64) -> Box<PageTable> {
        //how would you do this without a box? keeping 
        // in mind struct being on the stack and trying to get the address of L3 entries to put in L2
        let l2 = L2PageTable::new();
        let l3 = [L3PageTable::new(), L3PageTable::new()];
        let pagetable = PageTable {
            l2,
            l3
        };
        let mut boxed_pagetable = Box::new(pagetable);
        PageTable::initialize_l2_entries(&mut boxed_pagetable, perm, 2);

        boxed_pagetable

    }
    fn initialize_l2_entries(pagetable: &mut Box<PageTable>, perm: u64, num_entries: u16) {
        for i in (0..num_entries as usize) {   
            let current_entry = &mut pagetable.l2.entries[i];
            //set VALID to 1
            current_entry.set_value(0b1, RawL2Entry::VALID);
            //set TYPE to 1
            current_entry.set_value(0b1, RawL2Entry::TYPE);
            //set ATTR to 0b000 (normal memory)
            current_entry.set_value(0b000, RawL2Entry::ATTR);
            // set AP to 0b01 (User R/W)
            current_entry.set_value(perm, RawL2Entry::AP);
            // set SH to 0b11 (Inner shareable)
            current_entry.set_value(0b11, RawL2Entry::SH);
            //set AF to 1? (not sure what value and why do we set this when we create it instead of the first time we use it)
            current_entry.set_value(0b1, RawL2Entry::AF);

            //set ADDR to address of corresponding L3PageTable
            let entry_value_47_16 = & pagetable.l3[i] as *const L3PageTable as *const u64 as u64;
            current_entry.set_masked(entry_value_47_16, RawL2Entry::ADDR);

        }
    }
    
    /// Returns the (L2index, L3index) extracted from the given virtual address.
    /// Since we are only supporting 1GB virtual memory in this system, L2index
    /// should be smaller than 2.
    ///
    /// # Panics
    ///
    /// Panics if the virtual address is not properly aligned to page size.
    /// Panics if extracted L2index exceeds the number of L3PageTable.
    fn locate(va: VirtualAddr) -> (usize, usize) {
        let va_u64 = va.as_u64();
        if va_u64 % PAGE_SIZE as u64 != 0 { panic!("Virtual address not aligned to PAGE_SZIE"); }
        //13 bits to mask different parts of the VA
        let mask = 0b1_1111_1111_1111;
        // let l2_index = va_u64 & (mask << 29);
        let l2_index = (va_u64 & (0b1 << 29)) >> 29;
        let l3_index = (va_u64 & (mask << 16)) >> 16;
        if l2_index > 1 { panic!("L2 index can't be greater than 2"); }
        (l2_index as usize, l3_index as usize)
    }

    /// Returns `true` if the L3entry indicated by the given virtual address is valid.
    /// Otherwise, `false` is returned.
    pub fn is_valid(&self, va: VirtualAddr) -> bool {
        let (l2_index, l3_index) = PageTable::locate(va);
        self.l3[l2_index].entries[l3_index].is_valid()
    }

    /// Returns `true` if the L3entry indicated by the given virtual address is invalid.
    /// Otherwise, `true` is returned.
    pub fn is_invalid(&self, va: VirtualAddr) -> bool {
        !self.is_valid(va)
    }

    /// Set the given RawL3Entry `entry` to the L3Entry indicated by the given virtual
    /// address.
    pub fn set_entry(&mut self, va: VirtualAddr, entry: RawL3Entry) -> &mut Self {
        // are the directions for this method wrong?
        let (l2_index, l3_index) = PageTable::locate(va);
        self.l3[l2_index].entries[l3_index].0 = entry;
        self

    }

    /// Returns a base address of the pagetable. The returned `PhysicalAddr` value
    /// will point the start address of the L2PageTable.
    pub fn get_baddr(&self) -> PhysicalAddr {
        let l2_base_addr = & self.l2 as *const L2PageTable;
        From::from(l2_base_addr)
    }
}

// FIXME: Implement `IntoIterator` for `&PageTable`.
impl<'a> IntoIterator for &'a PageTable {
    type Item = &'a L3Entry;
    type IntoIter = Chain<Iter<'a, L3Entry>, Iter<'a, L3Entry>>;

    fn into_iter(self) -> Self::IntoIter {
        self.l3[0].entries.iter().chain(self.l3[1].entries.iter())
    }
}

pub struct KernPageTable(Box<PageTable>);

impl KernPageTable {
    /// Returns a new `KernPageTable`. `KernPageTable` should have a `Pagetable`
    /// created with `KERN_RW` permission.
    ///
    /// Set L3entry of ARM physical address starting at 0x00000000 for RAM and
    /// physical address range from `IO_BASE` to `IO_BASE_END` for peripherals.
    /// Each L3 entry should have correct value for lower attributes[10:0] as well
    /// as address[47:16]. Refer to the definition of `RawL3Entry` in `vmsa.rs` for
    /// more details.
    pub fn new() -> KernPageTable {
        let mut kernel_pagetable = PageTable::new(PagePerm::RW as u64); 
        // let limiting_factor = core::cmp::min(end_address, IO_BASE);
        let mut current_page_start = 0;
        let last_page_start = IO_BASE_END - PAGE_SIZE;
        //handle cases where end_address is greater than IO_BASE? is that possible
        while current_page_start <= last_page_start {
            if KernPageTable::is_valid_page(current_page_start) {
                let new_entry;
                if KernPageTable::is_normal_page(current_page_start) {
                    new_entry = KernPageTable::initialize_l3_entry(current_page_start, true);
                } else {
                    // guranteed that this is a device page given structure of conditionals
                    new_entry = KernPageTable::initialize_l3_entry(current_page_start, false);
                }
                kernel_pagetable.set_entry(VirtualAddr::from(current_page_start), new_entry);
            }
            current_page_start += PAGE_SIZE;
        }
        KernPageTable(kernel_pagetable)
    }

    fn is_normal_page(page_start: usize) -> bool {
        let (_, end_address) = allocator::memory_map().expect("Memory map failed");
        let last_normal_page_start = end_address - PAGE_SIZE;
        page_start >= 0 && page_start <= last_normal_page_start
    }
    fn is_device_page(page_start: usize) -> bool {
        //while loop constrains the page never being last_page_start = IO_BASE_END - PAGE_SIZE;
        page_start >= IO_BASE 
    }
    fn is_valid_page(page_start: usize) -> bool {
        KernPageTable::is_normal_page(page_start) || KernPageTable::is_device_page(page_start)
    }



    fn initialize_l3_entry(current_page: usize, is_normal_memory: bool) -> RawL3Entry {
            let mut entry = RawL3Entry::new(0);
            //set VALID to 1
            entry.set_value(0b1, RawL2Entry::VALID);
            //set TYPE to 1
            entry.set_value(0b1, RawL2Entry::TYPE);
            if is_normal_memory {
                //set ATTR to 0b000 (normal memory)
                entry.set_value(0b000, RawL2Entry::ATTR);
                // set SH to 0b11 (Inner shareable)
                entry.set_value(0b11, RawL2Entry::SH);
            } else { 
                //set ATTR to 0b001 (device memory)
                entry.set_value(0b001, RawL2Entry::ATTR);
                // set SH to 0b10 (Outer shareable)
                entry.set_value(0b10, RawL2Entry::SH);
            }
            // set AP to 0b00 (Kerne; R/W)
            entry.set_value(PagePerm::RW as u64, RawL2Entry::AP);
            //set AF to 1? (not sure what value and why do we set this when we create it instead of the first time we use it)
            entry.set_value(0b1, RawL2Entry::AF);
            //set ADDR to the upper bits of the page
            entry.set_masked(current_page as u64, RawL2Entry::ADDR);
            let (_, end) = allocator::memory_map().unwrap();
            
            entry

    }
}

pub enum PagePerm {
    RW,
    RO,
    RWX,
}

pub struct UserPageTable(Box<PageTable>);

impl UserPageTable {
    /// Returns a new `UserPageTable` containing a `PageTable` created with
    /// `USER_RW` permission.
    pub fn new() -> UserPageTable {
        let pagetable = PageTable::new(0b01);
        UserPageTable(pagetable)
    }

    /// Allocates a page and set an L3 entry translates given virtual address to the
    /// physical address of the allocated page. Returns the allocated page.
    ///
    /// # Panics
    /// Panics if the virtual address is lower than `USER_IMG_BASE`.
    /// Panics if the virtual address has already been allocated.
    /// Panics if allocator fails to allocate a page.
    ///
    /// TODO. use Result<T> and make it failurable
    /// TODO. use perm properly
    pub fn alloc(&mut self, va: VirtualAddr, _perm: PagePerm) -> &mut [u8] {
        let layout = Page::layout();
        let mut allocated_page_ptr;
        unsafe {
             allocated_page_ptr = ALLOCATOR.alloc(layout);
        }
        let allocated_page_addr = allocated_page_ptr as *const u64 as u64;
        let relative_va = va.as_usize() - USER_IMG_BASE;
        self.initialize_and_set_l3_entry(relative_va, allocated_page_addr);
        unsafe {
            //is this right? why are we returning an array of bytes? where is the return value ever used?
            let mut arr = core::slice::from_raw_parts_mut(allocated_page_ptr, PAGE_SIZE);
            // for i in (0..arr.len()) {
            //     arr[i] = 0;
            // }
            arr
        }
    }

    fn initialize_and_set_l3_entry(&mut self, va: usize, addr: u64) {
            let mut entry = RawL3Entry::new(0);
            //set VALID to 1
            entry.set_value(0b1, RawL2Entry::VALID);
            //set TYPE to 1
            entry.set_value(0b1, RawL2Entry::TYPE);
            //set ATTR to 0b000 (normal memory)
            entry.set_value(0b000, RawL2Entry::ATTR);
            // set AP to 0b01 (User R/W)
            entry.set_value(0b01, RawL2Entry::AP);
            // set SH to 0b11 (Inner shareable)
            entry.set_value(0b11, RawL2Entry::SH);
            //set AF to 1? (not sure what value and why do we set this when we create it instead of the first time we use it)
            entry.set_value(0b1, RawL2Entry::AF);
            //set ADDR to address of the page
            entry.set_masked(addr, RawL2Entry::ADDR);
            //set the RawL3Entry in the L3 table
            self.set_entry(VirtualAddr::from(va), entry);
    }
}

impl Deref for KernPageTable {
    type Target = PageTable;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for UserPageTable {
    type Target = PageTable;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for KernPageTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl DerefMut for UserPageTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// DONE: Implement `Drop` for `UserPageTable`.
impl Drop for UserPageTable {
    fn drop(&mut self) {
        for &l3_entry in self.into_iter() {
            if l3_entry.is_valid() {
                let mut physical_page_address = l3_entry.get_page_addr().expect("All pages should be bc of the conditional above");
                let physical_page_ptr = physical_page_address.as_mut_ptr();
                unsafe {
                    ALLOCATOR.dealloc(physical_page_ptr, Page::layout());
                }
            }
        }
    }
}
// FIXME: Implement `fmt::Debug` as you need.
impl fmt::Debug for UserPageTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "USerPageTable")
    }
}
