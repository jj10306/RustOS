// test4.rs
// This test covers the sections:
// - Modules
// - Macros

// Write a macro that passes the test! No hints this time, you can do it!

// I AM DONE

macro_rules! my_macro {
    ($s:expr) => {
        format!("Hello {}", $s)  
    };
}
fn main() {
    if my_macro!("world!") != "Hello world!" {
        panic!("Oh no! Wrong output!");
    }
}
