use log::info;
use simple_logger::SimpleLogger;
use std::borrow::Cow;
use std::env;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{self, BufRead};
use std::iter::Peekable;
use std::path::Path;

fn read_lines<P>(filename: P) -> io::Result<io::Split<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).split(b'\n'))
}

struct FloorMap {
    height: i32,
    width: i32,
    map: Vec<bool>,
}

impl FloorMap {
    fn new_from_file<P>(filename: P) -> FloorMap
    where
        P: AsRef<Path>,
    {
        let mut line_iter = read_lines(filename).unwrap().map(|res| res.unwrap());
        return Self::new_from_lines(&mut line_iter);
    }

    fn new_from_lines<I, S, T>(line_iter: T) -> FloorMap
    where
        I: Iterator<Item = S>,
        S: AsRef<[u8]>,
        T: IntoIterator<IntoIter = I, Item = S>,
    {
        let mut map = Vec::<bool>::new();
        let mut nlines = 0;

        let mut process_line = |line: &[u8], expected_width: Option<i32>| -> i32 {
            let mut w = 0;
            for c in line {
                map.push(match *c {
                    b'@' => true,
                    b'.' => false,
                    _ => {
                        panic!("Unexpected input {}", *c);
                    }
                });
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
        let width = process_line(first.as_ref(), None);
        nlines += 1;

        while let Some(line) = line_iter.next() {
            let w = process_line(line.as_ref(), Some(width));
            if w == 0 && line_iter.peek().is_none() {
                break;
            }
            if w != width {
                panic!("Mismatched line width {}, expected {}", w, width);
            }
            nlines += 1;
        }

        FloorMap {
            height: nlines,
            width: width,
            map: map,
        }
    }

    fn map_val(&self, x: i32, y: i32) -> bool {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            false
        } else {
            self.map[y as usize * self.width as usize + x as usize]
        }
    }

    fn free_val(&mut self, x: i32, y: i32) {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            panic!("out of bounds!");
        } else {
            self.map[y as usize * self.width as usize + x as usize] = false;
        }
    }

    fn count_neighbors(&self, x: i32, y: i32) -> u8 {
        if x < 0 {
            panic!("width {} below 0!", x);
        }
        if y < 0 {
            panic!("height {} below 0!", y);
        }
        if x >= self.width {
            panic!(
                "width {} exceeded width of map in FloorMap! {}",
                x, self.width
            );
        }
        if y >= self.height {
            panic!(
                "height {} exceeded # height of FloorMap! {}",
                y, self.height
            );
        }
        let mut sum = 0;
        for xoff in -1..2 {
            for yoff in -1..2 {
                sum += if xoff == 0 && yoff == 0 {
                    0
                } else {
                    // info!(
                    //     "x {} y {} eval {} {} -> {}",
                    //     x,
                    //     y,
                    //     x + xoff,
                    //     y + yoff,
                    //     self.map_val(x + xoff, y + yoff) as u8
                    // );
                    self.map_val(x + xoff, y + yoff) as u8
                };
            }
        }
        //info!("({}, {}) -> {}", x, y, sum);
        sum
    }

    fn count_free(&self, free_threshold: u8) -> u32 {
        let mut sum = 0;
        for x in 0..self.width {
            for y in 0..self.height {
                if self.map_val(x, y) {
                    sum += (self.count_neighbors(x, y) < free_threshold) as u32;
                }
            }
        }
        sum
    }

    fn count_and_mark_free(&mut self, free_threshold: u8) -> u32 {
        let mut sum = 0;
        for x in 0..self.width {
            for y in 0..self.height {
                if self.map_val(x, y) {
                    if self.count_neighbors(x, y) < free_threshold {
                        sum += 1;
                        self.free_val(x, y);
                    }
                }
            }
        }
        sum
    }

    fn count_and_mark_exhaust(&mut self, free_threshold: u8) -> u32 {
        let mut sum = 0;
        loop {
            let pass_sum = self.count_and_mark_free(free_threshold);
            if pass_sum == 0 {
                break;
            }
            sum += pass_sum;
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
    let mut map = FloorMap::new_from_file(file);
    println!("{}", map.count_and_mark_exhaust(4));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_map() {
        let b = FloorMap {
            width: 4,
            height: 3,
            #[rustfmt::skip]
            map: [
                false, false, true, false,
                true, false, false, true,
                true, false, true, true,
                true, false, true, true,
            ]
            .to_vec(),
        };
    }
    #[test]
    fn test_new_map_file() {
        let b = FloorMap::new_from_file("test.txt");
    }

    #[test]
    fn test_new_from_str() {
        let map = b"..@@.@@@@.
@@@.@.@.@@
@@@@@.@.@@
@.@@@@..@.
@@.@@@@.@@
.@@@@@@@.@
.@.@.@.@@@
@.@@@.@@@@
.@@@@@@@@.
@.@.@@@.@.";
        let mut b = FloorMap::new_from_lines(map.split(|&v| v == b'\n'));
        assert_eq!(b.count_neighbors(0, 0), 2);
        assert_eq!(b.count_free(4), 13);
        assert_eq!(b.count_and_mark_exhaust(4), 43);
    }
}
