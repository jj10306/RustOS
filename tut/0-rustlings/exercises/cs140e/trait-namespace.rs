// FIXME: Make me compile! Diff budget: 1 line.

// I AM NOT DONE

// Do not change this module.
use a::MyTrait;

mod a {
    pub trait MyTrait {
        fn foo(&self) {}
    }

    pub struct MyType;

    impl MyTrait for MyType {}
}

// Do not modify this function.
fn main() {
    let x = a::MyType;
    x.foo();
}
