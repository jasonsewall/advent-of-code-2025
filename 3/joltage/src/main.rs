use log::info;
use simple_logger::SimpleLogger;
use std::env;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{self, BufRead};
use std::iter::Peekable;
use std::path::Path;

fn read_lines<P>(filename: P) -> io::Result<Peekable<io::Split<io::BufReader<File>>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).split(b'\n').peekable())
}

struct BatteryBank {
    nbanks: u32,
    bank_width: u32,
    banks: Vec<u8>,
}

fn argmax(slice: &[u8]) -> usize {
    if slice.len() == 0 {
        panic!("Empty slice");
    }

    let mut argmax = 0;
    for (i, v) in slice.into_iter().enumerate() {
        if *v > slice[argmax] {
            argmax = i;
        }
    }
    argmax
}

impl BatteryBank {
    fn new<P>(filename: P) -> BatteryBank
    where
        P: AsRef<Path>,
    {
        let mut banks = Vec::<u8>::new();
        let mut nlines = 0;

        let mut process_line = |line: &[u8], expected_width: Option<u32>| -> u32 {
            let mut w = 0;
            for c in line {
                if *c <= b'0' || *c > b'9' {
                    panic!("Expected digit in [1-9], got {}", *c);
                }
                banks.push(*c - b'0');
                w += 1;
                if let Some(ewidth) = expected_width {
                    if w > ewidth {
                        panic!("Line exceeded expected width {}", ewidth);
                    }
                }
            }
            w
        };

        let mut line_iter = read_lines(filename).unwrap();
        let first = match line_iter.next() {
            Some(f) => f.unwrap(),
            None => panic!("No lines to read!"),
        };
        let width = process_line(&first, None);
        nlines += 1;

        while let Some(line) = line_iter.next() {
            let w = process_line(&line.unwrap(), Some(width));
            if w == 0 && line_iter.peek().is_none() {
                break;
            }
            if w != width {
                panic!("Mismatched line width {}, expected {}", w, width);
            }
            nlines += 1;
        }

        BatteryBank {
            nbanks: nlines,
            bank_width: width,
            banks: banks,
        }
    }

    fn bank_offset_val(&self, bankno: u32, offset: u32) -> u8 {
        if bankno > self.nbanks {
            panic!(
                "Bank # {} exceeded # of banks in BatteryBank! {}",
                bankno, self.nbanks
            );
        }
        if offset > self.bank_width {
            panic!(
                "Width {} exceeded # width of BatteryBank! {}",
                offset, self.bank_width
            );
        }
        self.banks[bankno as usize * self.bank_width as usize + offset as usize]
    }

    fn bank(&self, bankno: u32) -> &[u8] {
        if bankno > self.nbanks {
            panic!(
                "Bank # {} exceeded # of banks in BatteryBank! {}",
                bankno, self.nbanks
            );
        }
        let base = bankno as usize * self.bank_width as usize;
        &self.banks[base..(base + self.bank_width as usize)]
    }

    fn bank_max_joltage(&self, bankno: u32) -> u8 {
        let joltages = self.bank(bankno);

        let first_pos = argmax(&joltages[..joltages.len() - 1]);
        let second_pos = argmax(&joltages[first_pos + 1..]) + first_pos + 1;
        //info!("f {} s {} ", joltages[first_pos], joltages[second_pos]);
        joltages[first_pos] * 10_u8 + joltages[second_pos]
    }

    fn sum_max_joltages(&self) -> u32 {
        let mut sum = 0_u32;
        for b in 0..self.nbanks {
            sum += self.bank_max_joltage(b) as u32;
        }
        sum
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
    let bank = BatteryBank::new(file);

    println!("Max joltage is {}", bank.sum_max_joltages());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_bank() {
        let b = BatteryBank {
            bank_width: 4,
            nbanks: 2,
            banks: [1, 3, 3, 9, 2, 4, 1, 6].to_vec(),
        };
        assert_eq!(b.bank_max_joltage(0), 39);
        assert_eq!(b.bank_max_joltage(1), 46);
    }

    #[test]
    fn test_argmax() {
        let v = [1, 3, 3, 9, 2, 4, 1, 6].to_vec();
        assert_eq!(argmax(&v), 3);
        assert_eq!(argmax(&v[..3]), 1);
        assert_eq!(4 + argmax(&v[4..]), 7);
    }
}
