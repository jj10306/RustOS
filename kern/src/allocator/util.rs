/// Align `addr` downwards to the nearest multiple of `align`.
///
/// The returned usize is always <= `addr.`
///
/// # Panics
///
/// Panics if `align` is not a power of 2.
pub fn align_down(addr: usize, align: usize) -> usize {
    if align == 0 || (align & (align - 1)) != 0  {
        panic!("Align must be a power of 2!")
    }
    let divides = addr / align;
    if divides > 0 {
        align * divides
    } else {
        0   
    }
}

/// Align `addr` upwards to the nearest multiple of `align`.
///
/// The returned `usize` is always >= `addr.`
///
/// # Panics
///
/// Panics if `align` is not a power of 2
/// or aligning up overflows the address.
pub fn align_up(addr: usize, align: usize) -> usize {
    if align == 0 || (align & (align - 1)) != 0  {
        panic!("Align must be a power of 2!")
    }
    if addr == 0 {
        return 0
    }
    let divides = addr / align;
    if divides > 0 {
        if addr % align == 0 {
            align * divides
        } else {
            align * (divides + 1)
        }
    } else {
        align   
    }
}
