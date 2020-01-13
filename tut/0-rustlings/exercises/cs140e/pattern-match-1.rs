// FIXME: Make me compile! Diff budget: 2 lines.

// I AM DONE

// Do not change this definition.
enum MyEnum {
    A(String),
    B(String),
}

fn matcher(val: &MyEnum) -> &str {
    match &(*val) { //original is getting rid of &
        MyEnum::A(string) => string.as_str(),
        MyEnum::B(string) => string.as_str()
    }
}

fn main() {}
