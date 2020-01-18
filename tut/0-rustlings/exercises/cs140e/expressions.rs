// FIXME: Make me pass! Diff budget: 10 lines.
// Do not `use` any items.

// I AM DONE

// Do not change the following two lines.
#[derive(Debug, PartialOrd, PartialEq, Clone, Copy)]
struct IntWrapper(isize);

// Implement a generic function here
fn max<T>(num1: T, num2: T) -> T where T:PartialOrd {
    if num1 > num2 {
        return num1
    } else {
        return num2
    }
}

#[test]
fn expressions() {
    assert_eq!(max(1usize, 3), 3);
    assert_eq!(max(1u8, 3), 3);
    assert_eq!(max(1u8, 3), 3);
    assert_eq!(max(IntWrapper(120), IntWrapper(248)), IntWrapper(248));
}
