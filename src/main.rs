mod error;
use crate::error::Error;

use std::{
    env,
    fs::{self, File},
    io::{self, BufReader, BufWriter, Read, Write},
};

fn read_script() -> io::Result<Vec<u8>> {
    let filename = env::args().nth(1).expect("Please provide a file");
    let mut file = File::open(&filename)?;
    let mut script = Vec::with_capacity(fs::metadata(filename)?.len() as usize);
    file.read_to_end(&mut script)?;
    Ok(script)
}

#[derive(Debug, Default)]
#[cfg(feature = "true-infinite")]
struct Memory {
    vec: Vec<u8>,
    back: Vec<u8>,
    ptr: usize,
    back_ptr: usize,
    is_back: bool,
}

#[cfg(not(feature = "true-infinite"))]
struct Memory {
    vec: [u8; 30_000],
    ptr: usize,
}

#[cfg(not(feature = "true-infinite"))]
impl Default for Memory {
    fn default() -> Self {
        Memory {
            vec: [0; 30_000],
            ptr: 0,
        }
    }
}

impl Memory {
    #[allow(clippy::collapsible_if)]
    fn fill_to_ptr(&mut self) {
        #[cfg(feature = "true-infinite")]
        {
            if self.is_back && self.back_ptr >= self.back.len() {
                self.back.resize_with(self.back_ptr + 1, Default::default);
            } else if self.ptr >= self.vec.len() {
                self.vec.resize_with(self.ptr + 1, Default::default);
            }
        }
    }

    fn inc(&mut self) {
        #[cfg(feature = "true-infinite")]
        {
            if self.is_back {
                self.back_ptr = self.back_ptr.checked_sub(1).unwrap_or_else(|| {
                    self.is_back = false;
                    0
                });
            } else {
                self.ptr += 1;
            }
        }
        #[cfg(not(feature = "true-infinite"))]
        {
            self.ptr = self.ptr.wrapping_add(1);
        }
    }

    fn dec(&mut self) {
        #[cfg(feature = "true-infinite")]
        {
            if self.is_back {
                self.back_ptr += 1;
            } else {
                self.ptr = self.ptr.checked_sub(1).unwrap_or_else(|| {
                    self.is_back = true;
                    0
                });
            }
        }
        #[cfg(not(feature = "true-infinite"))]
        {
            self.ptr = self.ptr.wrapping_sub(1);
        }
    }

    fn plus(&mut self) {
        self.fill_to_ptr();
        #[cfg(feature = "true-infinite")]
        {
            if self.is_back {
                self.back[self.back_ptr] = self.back[self.back_ptr].wrapping_add(1);
            } else {
                self.vec[self.ptr] = self.vec[self.ptr].wrapping_add(1);
            }
        }
        #[cfg(not(feature = "true-infinite"))]
        {
            self.vec[self.ptr] = self.vec[self.ptr].wrapping_add(1);
        }
    }

    fn minus(&mut self) {
        self.fill_to_ptr();
        #[cfg(feature = "true-infinite")]
        {
            if self.is_back {
                self.back[self.back_ptr] = self.back[self.back_ptr].wrapping_sub(1);
            } else {
                self.vec[self.ptr] = self.vec[self.ptr].wrapping_sub(1);
            }
        }
        #[cfg(not(feature = "true-infinite"))]
        {
            self.vec[self.ptr] = self.vec[self.ptr].wrapping_sub(1);
        }
    }

    fn store(&mut self, byte: u8) {
        self.fill_to_ptr();
        #[cfg(feature = "true-infinite")]
        {
            if self.is_back {
                self.back[self.back_ptr] = byte;
            } else {
                self.vec[self.ptr] = byte;
            }
        }
        #[cfg(not(feature = "true-infinite"))]
        {
            self.vec[self.ptr] = byte;
        }
    }

    fn load(&mut self) -> u8 {
        self.fill_to_ptr();
        #[cfg(feature = "true-infinite")]
        {
            if self.is_back {
                self.back[self.back_ptr]
            } else {
                self.vec[self.ptr]
            }
        }
        #[cfg(not(feature = "true-infinite"))]
        {
            self.vec[self.ptr]
        }
    }

    fn debug(&mut self) -> bool {
        self.fill_to_ptr();
        #[cfg(feature = "true-infinite")]
        {
            let tape = format!(
                "{} back:{}",
                self.back
                    .iter()
                    .rev()
                    .chain(self.vec.iter())
                    .map(|x| x.to_string() + "|")
                    .collect::<String>(),
                self.is_back
            );
            eprintln!(
                "{}v p{} b{}",
                std::iter::repeat(' ')
                    .take(if self.is_back {
                        tape.match_indices('|')
                            .nth(self.back.len().checked_sub(self.back_ptr + 1).unwrap_or(0))
                            .map(|p| p.0 - 1)
                            .unwrap_or(0)
                    } else {
                        tape.match_indices('|')
                            .nth(self.back.len() + self.ptr)
                            .map(|p| p.0 - 1)
                            .unwrap_or(0)
                    })
                    .collect::<String>(),
                self.ptr,
                self.back_ptr
            );
            eprintln!("{}", tape);
        }
        #[cfg(not(feature = "true-infinite"))]
        {
            let tape = self
                .vec
                .iter()
                .map(|x| x.to_string() + "|")
                .collect::<String>();
            eprintln!(
                "{}v",
                std::iter::repeat(' ')
                    .take(
                        tape.match_indices('|')
                            .nth(self.ptr - 1)
                            .map(|p| p.0 - 1)
                            .unwrap_or(0)
                    )
                    .collect::<String>()
            );
            eprintln!("{}", tape);
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
        true
    }
}

pub fn interpret(bytes: Vec<u8>) -> Result<(), Error> {
    let sin = io::stdin();
    let mut stdin = BufReader::new(sin.lock());
    let sout = io::stdout();
    let mut stdout = BufWriter::new(sout.lock());
    let mut memory = Memory::default();
    let mut loop_counter = Vec::with_capacity(30);
    let mut ptr = 0;
    while ptr < bytes.len() {
        match bytes[ptr] {
            b'>' => memory.inc(),
            b'<' => memory.dec(),
            b'+' => memory.plus(),
            b'-' => memory.minus(),
            b'[' => {
                if memory.load() != 0 {
                    loop_counter.push(ptr)
                } else {
                    let mut skip_loop_counter = 1;
                    loop {
                        ptr += 1;
                        match bytes[ptr] {
                            b'[' => skip_loop_counter += 1,
                            b']' => skip_loop_counter -= 1,
                            _ => (),
                        }
                        if skip_loop_counter == 0 { break; }
                    };
                }
            }
            b']' => {
                if memory.load() != 0 {
                    ptr = loop_counter.pop().ok_or_else(|| Error::at_byte(ptr))? - 1;
                }
            }
            b',' => {
                let mut byte = [0];
                stdin.read_exact(&mut byte)?;
                memory.store(byte[0]);
            }
            b'.' => {
                let byte = [memory.load()];
                stdout.write_all(&byte)?;
            }
            _ => (),
        }
        debug_assert!(memory.debug());
        ptr += 1;
    }

    Ok(())
}

fn main() -> Result<(), Error> {
    interpret(read_script()?)
}
