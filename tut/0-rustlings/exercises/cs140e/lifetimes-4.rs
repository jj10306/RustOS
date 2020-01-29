// FIXME: Make me compile! Diff budget: 3 lines.

// I AM DONE

// Do not modify the inner type &'a T.
struct RefWrapper<'a, T>(&'a T);

// Do not modify the inner type &'b RefWrapper<'a, T>.
struct RefWrapperWrapper<'a, T>(&'a RefWrapper<'a, T>);

impl<'a, T> RefWrapperWrapper<'a, T> {
    fn inner(&self) -> &'a T {
        (self.0).0
    }
}

pub fn main() { }
