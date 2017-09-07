use std::num::Wrapping;
use std::io::{self, Read, Write, Bytes};

use ir::Atom;
use backend::Backend;

const MEM_SIZE: usize = 30_000;

#[derive(Debug)]
pub enum InterpreterError {
    IndexOutOfBounds(usize),
    EmptyInput,
    IOError(io::Error),
    LoopLimit,
}

#[derive(Debug)]
pub struct Interpreter<R: Read, W: Write> {
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
}

impl<R: Read, W: Write> Backend for Interpreter<R, W> {
    type Payload = ();
    type Error = InterpreterError;

    fn initialize(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn finalize(self) -> Result<Self::Payload, Self::Error> {
        Ok(())
    }

    fn push_move_ptr(&mut self, offset: isize) -> Result<(), Self::Error> {
        self.ptr = utils::offset_usize(self.ptr, offset) % MEM_SIZE;
        Ok(())
    }

    fn push_set_value(&mut self, value: i8, offset: isize) -> Result<(), Self::Error> {
        self.set_memory_offset(offset, Wrapping(value))
    }

    fn push_inc_value(&mut self, inc: i8, offset: isize) -> Result<(), Self::Error> {
        let old_value = self.get_memory_offset(offset)?;
        let new_value = old_value + Wrapping(inc);
        self.set_memory_offset(offset, new_value)
    }

    fn push_print(&mut self, offset: isize) -> Result<(), Self::Error> {
        let to_write = self.get_memory_offset(offset)?.0 as u8;
        self.writer
            .write(&[to_write])
            .map_err(|err| InterpreterError::IOError(err))
            .map(|_| ())
    }

    fn push_read(&mut self, offset: isize) -> Result<(), Self::Error> {
        if let Some(next) = self.reader.next() {
            match next {
                Ok(c) => {
                    self.set_memory_offset(offset, Wrapping(c as i8))?;
                    Ok(())
                },
                Err(err) => {
                    Err(InterpreterError::IOError(err))
                }, 
            }
        } else {
            Err(InterpreterError::EmptyInput)
        }
    }

    fn push_multiply(&mut self, factor: i8, offset: isize) -> Result<(), Self::Error> {
        let old_value = self.get_memory_offset(offset)?;
        let zero_value = self.get_memory_offset(0)?;
        let new_value = old_value + zero_value * Wrapping(factor);
        self.set_memory_offset(offset, new_value)
    }

    fn push_loop(&mut self, sub: &Vec<Atom>) -> Result<(), Self::Error> {
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
            self.push_atoms(sub)?;
        }
        Ok(())
    }
}

mod utils {
    pub fn offset_usize(base: usize, offset: isize) -> usize {
        if offset < 0 {
            base.wrapping_sub((-offset) as usize)
        } else {
            base.wrapping_add(offset as usize)
        }
    }
}
