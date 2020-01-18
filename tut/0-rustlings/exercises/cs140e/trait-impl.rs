// FIXME: Make me pass! Diff budget: 25 lines.

// I AM NOT DONE

#[derive(Debug)]
enum Duration {
    MilliSeconds(u64),
    Seconds(u32),
    Minutes(u16),
}
impl PartialEq for Duration {
    fn eq(&self, other: &Self) -> bool {
        let sec1 = match *self {
            Duration::MilliSeconds(ms) => ((ms as f64) / 1000f64),
            Duration::Seconds(s) => (s as f64),
            Duration::Minutes(m) => ((m * 60) as f64)
        };
        let sec2 = match *other {
            Duration::MilliSeconds(ms) => ((ms as f64) / 1000f64),
            Duration::Seconds(s) => (s as f64),
            Duration::Minutes(m) => ((m * 60) as f64)
        };
        sec1 == sec2
    }
}
// What traits does `Duration` need to implement?

#[test]
fn traits() {
    assert_eq!(Duration::Seconds(120), Duration::Minutes(2));
    assert_eq!(Duration::Seconds(420), Duration::Minutes(7));
    assert_eq!(Duration::MilliSeconds(420000), Duration::Minutes(7));
    assert_eq!(Duration::MilliSeconds(43000), Duration::Seconds(43));
}
