mod error;
use crate::error::Error;

use std::env;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};

fn read_script() -> io::Result<String> {
    let mut file = File::open(env::args().nth(1).expect("Please provide a file."))?;
    let mut script = String::new();
    file.read_to_string(&mut script)?;
    Ok(script)
}

#[derive(Debug, Default)]
struct Memory {
    vec: Vec<isize>,
    ptr: usize,
}

impl Memory {
    fn fill_to_ptr(&mut self) {
        while self.ptr >= self.vec.len() {
            self.vec.push(0)
        }
    }

    fn inc(&mut self) {
        self.ptr += 1;
    }

    fn dec(&mut self) {
        self.ptr = self.ptr.checked_sub(1).unwrap_or_else(|| {
            self.vec.insert(0, 0);
            0
        });
    }

    fn plus(&mut self) {
        self.fill_to_ptr();
        self.vec[self.ptr] += 1;
    }

    fn minus(&mut self) {
        self.fill_to_ptr();
        self.vec[self.ptr] -= 1;
    }

    fn store(&mut self, byte: u8) {
        self.fill_to_ptr();
        self.vec[self.ptr] = byte as isize;
    }

    fn load(&mut self) -> isize {
        self.fill_to_ptr();
        self.vec[self.ptr]
    }

    fn debug(&self) -> bool {
        eprintln!(
            "{}v",
            std::iter::repeat(' ')
                .take(
                    self.vec
                        .iter()
                        .take(self.ptr)
                        .map(|x| x.to_string().len() + 1)
                        .sum()
                )
                .collect::<String>()
        );
        eprintln!(
            "{}",
            self.vec
                .iter()
                .map(|x| x.to_string() + "|")
                .collect::<String>()
        );
        std::thread::sleep(std::time::Duration::from_millis(500));
        true
    }
}

fn main() -> Result<(), Error> {
    let sin = io::stdin();
    let mut stdin = BufReader::new(sin.lock());
    let sout = io::stdout();
    let mut stdout = BufWriter::new(sout.lock());

    let mut memory = Memory::default();
    let mut loop_counter = Vec::new();
    let bytes = read_script()?.chars().collect::<Vec<char>>();
    let mut ptr = 0;
    while ptr < bytes.len() {
        match bytes[ptr] {
            '>' => memory.inc(),
            '<' => memory.dec(),
            '+' => memory.plus(),
            '-' => memory.minus(),
            '[' => {
                if memory.load() != 0 {
                    loop_counter.push(ptr)
                }
            }
            ']' => {
                if memory.load() != 0 {
                    ptr = loop_counter.pop().ok_or_else(|| Error::at_byte(ptr))? - 1;
                }
            }
            ',' => {
                let mut byte = [0];
                stdin.read_exact(&mut byte)?;
                memory.store(byte[0]);
            }
            '.' => {
                let byte = memory.load();
                stdout.write_fmt(format_args!("{}", byte))?;
            }
            _ => (),
        }
        debug_assert!(memory.debug());
        ptr += 1;
    }

    Ok(())
}
