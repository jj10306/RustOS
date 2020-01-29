// FIXME: Make me pass! Diff budget: 2 lines.

// I AM DONE

// What traits does this struct need to derive?
#[derive(Debug, PartialEq)]
struct MyType(usize);

fn borrow2() {
    let mut x = MyType(1);
    let y = &mut x;

    // Do not modify this line.
    y.0 = 2;
    assert_eq!(*y, MyType(2));
}