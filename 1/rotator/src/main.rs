use log::info;
use simple_logger::SimpleLogger;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn parse_line(string: &str) -> Option<i32> {
    let first = string.chars().nth(0)?;
    let sign = match first {
        'R' => 1,
        'L' => -1,
        _ => return None,
    };

    match string[1..].parse::<i32>() {
        Ok(n) => return Some(sign * n),
        Err(e) => panic!("Not a number!"),
    };
}

struct Dial {
    state: i32,
    zero_ct: i32,
}
/*
state + n >= size
state - n <= 0

*/
impl Dial {
    fn spin(&mut self, n: i32) {
        let size = 100;
        let div = n / size;
        let min_n = n - div * size;
        let unmod = self.state + min_n;
        let oldstate = self.state;
        self.state = (self.state + min_n) % size;
        if self.state < 0 {
            self.state = size + self.state;
        }

        let mut cross = div.abs();
        if self.state == 0 && n != 0 {
            cross += 1;
        } else if oldstate > 0 && unmod < 0 {
            cross += 1;
        } else if unmod >= size {
            cross += 1;
        }
        info!(
            "state: {}, n: {}, div: {}, unmod: {}, state': {}, cross: {}",
            oldstate, n, div, unmod, self.state, cross
        );

        self.zero_ct += cross;
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
    info!("Opening {file}");

    let mut dial = Dial {
        state: 50,
        zero_ct: 0,
    };
    for line in read_lines(file).unwrap() {
        match parse_line(&line.unwrap()) {
            Some(n) => {
                dial.spin(n);
            }
            None => {
                continue;
            }
        }
    }
    println!("Zero count: {}", dial.zero_ct);
    Ok(())
}
