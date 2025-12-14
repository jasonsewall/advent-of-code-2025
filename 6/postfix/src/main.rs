use log::info;
use simple_logger::SimpleLogger;
use std::borrow::Cow;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

#[derive(Debug, PartialEq)]
struct InvalidAsciiU64;

fn ascii_to_u64(bytes: &[u8]) -> Result<(u64, usize), InvalidAsciiU64> {
    let mut end = 0;
    while end < bytes.len() {
        if bytes[end] < b'0' || bytes[end] > b'9' {
            break;
        }
        end += 1;
    }
    let end = end;
    let mut res = 0;
    let mut idx = 0;
    while idx < end {
        let val = (bytes[idx] - b'0') as u64;
        res += val * 10_u64.pow((end - idx - 1) as u32);
        idx += 1;
    }
    if idx == 0 {
        Err(InvalidAsciiU64)
    } else {
        Ok((res, idx))
    }
}

fn consume_space(bytes: &[u8]) -> usize {
    let mut c = 0;
    while c < bytes.len() && bytes[c] == b' ' {
        c += 1;
    }
    c
}

enum MathOp {
    Sum,
    Product,
}

enum LineType {
    Numbers,
    Ops,
    Empty,
}

fn classify_line_type(bytes: &[u8]) -> Result<LineType, UnknownLineType> {
    let idx = consume_space(bytes);
    if bytes[idx] >= b'0' && bytes[idx] <= b'9' {
        Ok(LineType::Numbers)
    } else if let Ok(_) = get_op(bytes[idx]) {
        Ok(LineType::Ops)
    } else if bytes.len() - idx == 0 {
        Ok(LineType::Empty)
    } else {
        Err(UnknownLineType)
    }
}

#[derive(Debug, PartialEq)]
struct UnknownLineType;

#[derive(Debug, PartialEq)]
struct InvalidMathOp(u8);

fn get_op(c: u8) -> Result<MathOp, InvalidMathOp> {
    match c {
        b'+' => Ok(MathOp::Sum),
        b'*' => Ok(MathOp::Product),
        _ => Err(InvalidMathOp(c)),
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Split<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).split(b'\n'))
}

struct MathProblems {
    height: u32,
    width: u32,
    numbers: Vec<u64>,
    operators: Vec<MathOp>,
}

impl MathProblems {
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
        let mut numbers = Vec::<u64>::new();
        let mut nlines = 0;

        let mut process_num_line = |line: &[u8], expected_width: Option<u32>| -> u32 {
            let mut w = 0;
            let mut idx = 0;
            while idx < line.len() {
                idx += consume_space(&line[idx..]);
                if let Ok((num, offs)) = ascii_to_u64(line[idx..].as_ref()) {
                    numbers.push(num);
                    //                    info!("Got Num: {} {} -> {}", num, idx, offs);
                    idx += offs;
                } else {
                    //                  info!("giving up on: {}", line[idx]);
                    break;
                }

                w += 1;
                if let Some(ewidth) = expected_width {
                    if w > ewidth {
                        panic!("Line exceeded expected width {}", ewidth);
                    }
                }
            }
            w
        };

        let mut ops = Vec::<MathOp>::new();

        let mut process_op_line = |line: &[u8], expected_width: Option<u32>| -> u32 {
            let mut w = 0;
            let mut idx = 0;
            loop {
                idx += consume_space(&line[idx..]);
                if idx < line.len() {
                    if let Ok(op) = get_op(line[idx]) {
                        ops.push(op);
                        idx += 1;
                    } else {
                        break;
                    }
                } else {
                    break;
                }

                w += 1;
                if let Some(ewidth) = expected_width {
                    if w > ewidth {
                        panic!("Line exceeded expected width {}", ewidth);
                    }
                }
            }
            w
        };

        let mut line_iter = line_iter.into_iter().peekable();

        let first = match line_iter.next() {
            Some(f) => f,
            None => panic!("No lines to read!"),
        };
        let width = match classify_line_type(first.as_ref()) {
            Ok(LineType::Numbers) => process_num_line(first.as_ref(), None),
            Ok(LineType::Ops) => {
                panic!("Got an ops line as first line!");
            }
            Ok(LineType::Empty) => {
                panic!("Got an empty line as first line!");
            }
            Err(UnknownLineType) => {
                panic!("Unknown line type!");
            }
        };
        nlines += 1;

        while let Some(line) = line_iter.next() {
            if let Ok(line_type) = classify_line_type(line.as_ref()) {
                match line_type {
                    LineType::Numbers => {
                        let w = process_num_line(line.as_ref(), Some(width));
                        if w != width {
                            panic!("Mismatched line width {}, expected {}", w, width);
                        }
                    }
                    LineType::Ops => {
                        let w = process_op_line(line.as_ref(), Some(width));
                        if w != width {
                            panic!("Mismatched line width {}, expected {}", w, width);
                        }
                    }
                    LineType::Empty => {
                        assert!(line_iter.peek().is_none() && ops.len() == width as usize);
                        break;
                    }
                }

                nlines += 1;
            } else {
                panic!("Unknown line type!");
            }
        }
        assert!(ops.len() == width as usize);

        Self {
            height: nlines - 1,
            width: width,
            numbers: numbers,
            operators: ops,
        }
    }

    fn solve(&self) -> Vec<u64> {
        let mut res: Vec<u64> = vec![0_u64; self.width as usize];
        for p in 0..(self.width as usize) {
            match self.operators[p] {
                MathOp::Sum => {
                    res[p] = 0;
                }
                MathOp::Product => {
                    res[p] = 1;
                }
            }
        }
        assert!(res.len() == self.width as usize);
        for h in 0..(self.height as usize) {
            for p in 0..(self.width as usize) {
                let val = self.numbers[h * (self.width as usize) + p];
                match self.operators[p] {
                    MathOp::Sum => {
                        res[p] += val;
                    }
                    MathOp::Product => {
                        res[p] *= val;
                    }
                }
            }
        }
        res
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
    let mathops = MathProblems::new_from_file(file);
    println!("width: {} height: {}", mathops.width, mathops.height);
    let v = mathops.solve();
    for (i, p) in v.iter().enumerate() {
        println!("{}: {}", i, p);
    }

    println!("Sum: {}", v.into_iter().sum::<u64>());
    Ok(())
}
