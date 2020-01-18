// FIXME: Make me compile! Diff budget: 1 line.

// I AM DONE

// What traits does this struct need to derive?
#[derive(Clone, Copy)]
struct MyType(usize);

// Do not modify this function.
pub fn main() {
    let x = MyType(1);
    let y = &x;
    let z = *y;
}
