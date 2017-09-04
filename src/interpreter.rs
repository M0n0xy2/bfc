use std::num::Wrapping;
use std::io::{self, Read, Write, Bytes};

use utils;
use ir::Atom;

pub fn interpret<R: Read, W: Write>(ir: &Vec<Atom>, reader: R, writer: W) -> Result<(), InterpreterError> {
    let mut int = Interpreter::new(reader, writer, None);
    int.interpret(ir)
}

pub fn interpret_with_loop_limit<R: Read, W: Write>(ir: &Vec<Atom>, reader: R, writer: W, loop_limit: usize) -> Result<(), InterpreterError> {
    let mut int = Interpreter::new(reader, writer, Some(loop_limit));
    int.interpret(ir)
}

#[derive(Debug)]
pub enum InterpreterError {
    IndexOutOfBounds(usize),
    EmptyInput,
    IOError(io::Error),
    LoopLimit,
}

const MEM_SIZE: usize = 30_000;

#[derive(Debug)]
struct Interpreter<R: Read, W: Write> {
    memory: Vec<Wrapping<i8>>,
    ptr: usize,
    loop_limit: Option<usize>,
    reader: Bytes<R>,
    writer: W,
}


impl<R: Read, W: Write> Interpreter<R, W> {
    pub fn new(reader: R, writer: W, loop_limit: Option<usize>) -> Self {
        Interpreter {
            memory: vec![Wrapping(0); MEM_SIZE],
            ptr: 0,
            loop_limit,
            reader: reader.bytes(),
            writer,
        }
    }

    fn set_memory_offset(&mut self, offset: isize, value: Wrapping<i8>) -> Result<(), InterpreterError> {
        let ptr = utils::offset_usize(self.ptr, offset) % MEM_SIZE;
        if let Some(cell) = self.memory.get_mut(ptr) {
            *cell = value;
            Ok(())
        } else {
            Err(InterpreterError::IndexOutOfBounds(ptr))
        }
    }

    fn get_memory_offset(&self, offset: isize) -> Result<Wrapping<i8>, InterpreterError> {
        let ptr = utils::offset_usize(self.ptr, offset) % MEM_SIZE;
        if let Some(cell) = self.memory.get(ptr) {
            Ok(*cell)
        } else {
            Err(InterpreterError::IndexOutOfBounds(ptr))
        }
    }

    fn interpret(&mut self, ir: &Vec<Atom>) -> Result<(), InterpreterError> {
        for atom in ir {
            match *atom {
                Atom::MovePtr(offset) => {
                    self.ptr = utils::offset_usize(self.ptr, offset) % MEM_SIZE;
                },
                Atom::SetValue(value, offset) => {
                    self.set_memory_offset(offset, Wrapping(value))?;
                },
                Atom::IncValue(inc, offset) => {
                    let old_value = self.get_memory_offset(offset)?;
                    let new_value = old_value + Wrapping(inc);
                    self.set_memory_offset(offset, new_value)?;
                },
                Atom::Print(offset) => {
                    let to_write = self.get_memory_offset(offset)?.0 as u8;
                    self.writer
                        .write(&[to_write])
                        .unwrap();
                },
                Atom::Read(offset) => {
                    if let Some(next) = self.reader.next() {
                        match next {
                            Ok(c) => {
                                self.set_memory_offset(offset, Wrapping(c as i8))?;
                            },
                            Err(err) => {
                                return Err(InterpreterError::IOError(err));
                            }, 
                        }
                    } else {
                        return Err(InterpreterError::EmptyInput);
                    }
                },
                Atom::Multiply(factor, offset) => {
                    let old_value = self.get_memory_offset(offset)?;
                    let zero_value = self.get_memory_offset(0)?;
                    let new_value = old_value + zero_value * Wrapping(factor);
                    self.set_memory_offset(offset, new_value)?;
                },
                Atom::Loop(ref sub) => {
                    let mut loop_counter = 0;
                    while self.get_memory_offset(0)?.0 != 0 {
                        // checking the loop limiter
                        loop_counter += 1;
                        if let Some(loop_limit) = self.loop_limit {
                            if loop_counter >= loop_limit {
                                return Err(InterpreterError::LoopLimit);
                            }
                        }

                        // interpreting the loop
                        self.interpret(sub)?;
                    }
                }
            }
        }
        Ok(())
    }
}
