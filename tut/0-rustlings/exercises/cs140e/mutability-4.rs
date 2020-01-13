// FIXME: Make me compile! Diff budget: 2 lines.

// I AM DONE

struct MyStruct(usize);

impl MyStruct {
    fn make_1(&mut self) { //self: &mut Self
        self.0 = 1;
    }
}

pub fn main() {
    let mut x = MyStruct(10);
    x.make_1();
}
