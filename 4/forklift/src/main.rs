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

struct FloorMap {
    height: u32,
    width: u32,
    map: Vec<bool>,
}

impl FloorMap {
    fn new_from_file<P>(filename: P) -> FloorMap
    where
        P: AsRef<Path>,
    {
        let line_iter = read_lines(filename)
            .unwrap()
            .map(|res| res.unwrap().as_ref());
        return Self::new_from_lines(line_iter);
    }

    fn new_from_lines<'a, I>(line_iter: &mut std::iter::Peekable<I>) -> FloorMap
    where
        I: Iterator<Item = &'a [u8]>,
    {
        let mut map = Vec::<bool>::new();
        let mut nlines = 0;

        let mut process_line = |line: &[u8], expected_width: Option<u32>| -> u32 {
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

        let first = match line_iter.next() {
            Some(f) => f,
            None => panic!("No lines to read!"),
        };
        let width = process_line(&first, None);
        nlines += 1;

        while let Some(line) = line_iter.next() {
            let w = process_line(&line, Some(width));
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

    fn map_val(&self, x: u32, y: u32) -> bool {
        if x > self.width {
            panic!(
                "width {} exceeded width of map in FloorMap! {}",
                x, self.width
            );
        }
        if y > self.height {
            panic!(
                "height {} exceeded # height of FloorMap! {}",
                y, self.height
            );
        }
        self.map[y as usize * self.width as usize + x as usize]
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
    let map = FloorMap::new(file);

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
        let b = FloorMap::new("test.txt");
    }
}
