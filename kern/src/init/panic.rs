use core::panic::PanicInfo;
use crate::console::kprintln;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {    
    let location = _info.location().unwrap();   

    kprintln!("------------- PANIC ---------------\n");
    kprintln!("FILE: {}", location.file());
    kprintln!("LINE: {}", location.line());
    kprintln!("COLUMN: {}", location.column());
    kprintln!("\n{}", _info.message().unwrap());
    loop {}
}
