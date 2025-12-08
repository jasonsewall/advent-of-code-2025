use log::info;
use simple_logger::SimpleLogger;
use std::borrow::Cow;
use std::env;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{self, BufRead};
use std::iter::Peekable;
use std::mem;
use std::path::Path;

#[derive(Debug, PartialEq)]
struct InvalidAsciiU64;

fn ascii_to_u64(bytes: &[u8]) -> Result<u64, InvalidAsciiU64> {
    let mut res = 0;
    for (i, c) in bytes.iter().enumerate() {
        if *c < b'0' || *c > b'9' {
            return Err(InvalidAsciiU64);
        }
        let val = (*c - b'0') as u64;
        res += val * 10_u64.pow((bytes.len() - i - 1) as u32);
    }
    Ok(res)
}

mod interval {
    use super::ascii_to_u64;
    use std::cmp::Ordering;

    #[derive(Debug, PartialEq)]
    pub struct InvalidClosedInt;

    #[derive(Debug, PartialEq)]
    pub struct UnmergableInts;

    #[derive(Debug, Eq, PartialEq, Clone)]
    pub struct ClosedInt {
        low: u64,
        high: u64,
    }

    impl ClosedInt {
        pub fn new(low: u64, high: u64) -> Result<Self, InvalidClosedInt> {
            if low > high {
                Err(InvalidClosedInt)
            } else {
                Ok(ClosedInt {
                    low: low,
                    high: high,
                })
            }
        }
        pub fn from_str(txt: &[u8]) -> Result<Self, InvalidClosedInt> {
            let mut idx = 0;
            loop {
                if idx >= txt.len() {
                    return Err(InvalidClosedInt);
                }
                if txt[idx] == b'-' {
                    break;
                }
                idx += 1;
            }
            let low = match ascii_to_u64(&txt[0..idx]) {
                Ok(low) => low,
                Err(_) => {
                    return Err(InvalidClosedInt);
                }
            };
            let high = match ascii_to_u64(&txt[idx + 1..]) {
                Ok(high) => high,
                Err(_) => {
                    return Err(InvalidClosedInt);
                }
            };
            ClosedInt::new(low, high)
        }

        pub fn is_before(&self, num: u64) -> bool {
            num > self.high
        }

        pub fn is_after(&self, num: u64) -> bool {
            num < self.low
        }

        pub fn merge(&self, other: &Self) -> Result<ClosedInt, UnmergableInts> {
            if self.high < other.low || other.low < self.high {
                Err(UnmergableInts)
            } else {
                Ok(ClosedInt::new(
                    std::cmp::min(self.low, other.low),
                    std::cmp::max(self.low, other.high),
                )
                .unwrap())
            }
        }

        pub fn contains(&self, num: u64) -> bool {
            num >= self.low && num <= self.high
        }
    }

    impl Ord for ClosedInt {
        fn cmp(&self, other: &Self) -> Ordering {
            self.low.cmp(&other.low)
        }
    }

    impl PartialOrd for ClosedInt {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(&other))
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_invalid() {
            assert_eq!(ClosedInt::new(16, 15), Err(InvalidClosedInt));
        }

        #[test]
        fn test_single() {
            if let Ok(x) = ClosedInt::new(15, 15) {
                assert_eq!(x.low, 15);
                assert_eq!(x.high, 15);
            } else {
                panic!("Invalid interval!");
            }
        }

        #[test]
        fn test_u8() {
            assert_eq!(ClosedInt::from_str(b"3-5"), ClosedInt::new(3, 5));
        }

        #[test]
        fn test_contains() {
            let closed = ClosedInt::new(10, 15).unwrap();
            for i in 10..15 {
                assert!(closed.contains(i));
            }
            assert!(!closed.contains(9));
            assert!(!closed.contains(16));
        }

        #[test]
        fn test_ord() {
            let closed0 = ClosedInt::new(10, 15).unwrap();
            let closed0_copy = ClosedInt::new(10, 15).unwrap();
            let closed1 = ClosedInt::new(20, 25).unwrap();
            assert!(closed0 < closed1);
            assert!(!(closed0 > closed1));
            assert!(closed0 != closed1);
            assert!(closed0 == closed0_copy);
        }
    }
}

use interval::ClosedInt;

fn bruteforce_interval(val: u64, intervals: &[ClosedInt]) -> bool {
    for i in intervals {
        if i.contains(val) {
            return true;
        }
    }
    false
}

fn pivot_intervals(intervals: &mut [ClosedInt]) -> &mut [ClosedInt] {
    if intervals.len() == 1 {
        return intervals;
    }

    let mut before = 0;
    let mut after = intervals.len() - 2;

    let mid = intervals.len() / 2;
    intervals.swap(mid, intervals.len() - 1);

    while before < after {
        if intervals[front].is_before(pivot) {
            continue;
            front += 1;
        } else if intervals[front].is_after(pivot) {
            intervals.swap(front, after);
            front += 1;
            after -= 1;
        } else {
            front += 1;
        }
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Split<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).split(b'\n'))
}

struct FoodbProblem {
    intervals: Vec<ClosedInt>,
    to_check: Vec<u64>,
}

impl FoodbProblem {
    fn new_from_file<P>(filename: P) -> Self
    where
        P: AsRef<Path>,
    {
        let mut line_iter = read_lines(filename).unwrap().map(|res| res.unwrap());
        return Self::new_from_lines(&mut line_iter);
    }

    fn new_from_lines<I, S, T>(line_iter: T) -> Self
    where
        I: Iterator<Item = S>,
        S: AsRef<[u8]>,
        T: IntoIterator<IntoIter = I, Item = S>,
    {
        let mut ints = Vec::<ClosedInt>::new();
        let mut line_iter = line_iter.into_iter().peekable();

        while let Some(line) = line_iter.next() {
            if line.as_ref().len() == 0 {
                break;
            }
            if let Ok(x) = ClosedInt::from_str(line.as_ref()) {
                ints.push(x);
            } else {
                panic!(
                    "Couldn't parse {} as interval",
                    std::str::from_utf8(line.as_ref()).expect("not utf8")
                );
            }
        }

        let mut ids = Vec::<u64>::new();

        while let Some(line) = line_iter.next() {
            if line.as_ref().len() == 0 {
                if line_iter.peek().is_none() {
                    break;
                } else {
                    panic!("Unexpected end of while parsing!");
                }
            }
            if let Ok(x) = ascii_to_u64(line.as_ref()) {
                ids.push(x);
            } else {
                panic!(
                    "Couldn't parse {} as id",
                    std::str::from_utf8(line.as_ref()).expect("not utf8")
                );
            }
        }

        ints.sort();
        FoodbProblem {
            intervals: ints,
            to_check: ids,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    SimpleLogger::new().init().unwrap();
    let mut args = env::args().skip(1).peekable();
    let file = match args.next() {
        Some(file) => file,
        None => {
            return Err(From::from("Need a file argument!"));
        }
    };
    let fdb = FoodbProblem::new_from_file(file);
    let mut res = 0;
    for c in fdb.to_check {
        res += bruteforce_interval(c, &fdb.intervals) as u64;
    }
    println!("{}", res);

    Ok(())
}
mod tests {
    use super::*;

    #[test]
    fn test_ascii_to_u64() {
        assert_eq!(ascii_to_u64(b"123123"), Ok(123123));
    }

    #[test]
    fn test_load() {
        let lines = b"3-5
10-14
16-20
12-18

1
5
8
11
17
32";
        let fdb = FoodbProblem::new_from_lines(lines.split(|&v| v == b'\n'));
        let mut res = 0;
        for c in fdb.to_check {
            res += bruteforce_interval(c, &fdb.intervals) as u64;
        }
        assert_eq!(res, 3);
    }
}
