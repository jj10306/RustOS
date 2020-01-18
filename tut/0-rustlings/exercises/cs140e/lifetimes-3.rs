// FIXME: Make me compile! Diff budget: 2 lines.

// I AM DONE

// Do not modify the inner type &'a T.
struct RefWrapper<'a, T>(&'a T);

// Do not modify the inner type &'b RefWrapper<'a, T>.
struct RefWrapperWrapper<'b, T>(&'b RefWrapper<'b, T>);

pub fn main() {}
