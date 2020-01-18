// FIXME: Make me pass! Diff budget: 30 lines.

// I AM DONE

#[derive(Default, Debug)]
struct Builder {
    string: Option<String>,
    number: Option<usize>,
}

impl Builder {
    fn string(mut self, aStr: &str) -> Self {
        let string = String::from(aStr);
        self.string = Some(string);
        self
    }

    fn number(mut self, aNum: usize) -> Self {
        self.number = Some(aNum);
        self
    }
}

impl ToString for Builder {
    fn to_string(&self) -> String {
        match self {
            Builder { string: Some(aStr), number: Some(aNum) } => format!("{} {:?}", aStr, aNum),
            Builder { string: Some(aStr), number: None } => aStr.to_string(),
            Builder { string: None, number: Some(aNum) } => format!("{:?}", aNum),
            Builder { string: None, number: None } => "".to_string()
        }
        
    }
}

// Do not modify this function.
#[test]
fn builder() {
    let empty = Builder::default().to_string();
    assert_eq!(empty, "");

    let just_str = Builder::default().string("hi").to_string();
    assert_eq!(just_str, "hi");

    let just_num = Builder::default().number(254).to_string();
    assert_eq!(just_num, "254");

    let a = Builder::default()
        .string("hello, world!")
        .number(200)
        .to_string();

    assert_eq!(a, "hello, world! 200");

    let b = Builder::default()
        .string("hello, world!")
        .number(200)
        .string("bye now!")
        .to_string();

    assert_eq!(b, "bye now! 200");

    let c = Builder::default().string(&"heap!".to_owned()).to_string();

    assert_eq!(c, "heap!");
}
