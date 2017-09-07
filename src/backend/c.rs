use std::io::{self, Write};

use ir::Atom;
use backend::Backend;

const MEM_SIZE: usize = 30000;

#[derive(Debug, Clone)]
pub struct CBackend<W: Write> {
    writer: W,
    current_tab: usize,
}

impl<W: Write> CBackend<W> {
    pub fn new(writer: W) -> Self {
        CBackend {
            writer,
            current_tab: 1,
        }
    }

    fn write_tab(&mut self) -> io::Result<()> {
        write!(&mut self.writer, "{}", "\t".repeat(self.current_tab))
    }
}

impl<W: Write> Backend for CBackend<W> {
    type Payload = ();
    type Error = io::Error;

    fn initialize(&mut self) -> Result<(), Self::Error> {
        writeln!(&mut self.writer, "#include <stdlib.h>")?;
        writeln!(&mut self.writer, "#include <stdio.h>")?;
        writeln!(&mut self.writer, "#include <stdint.h>")?;

        writeln!(&mut self.writer, "int8_t memory[{}];", MEM_SIZE)?;
        writeln!(&mut self.writer, "int8_t* ptr = memory;")?;
        writeln!(&mut self.writer, "int main() {{")?;
        Ok(())
    }

    fn finalize(mut self) -> Result<(), Self::Error> {
        writeln!(&mut self.writer, "}}")
    }

    fn add_move_ptr(&mut self, offset: isize) -> Result<(), Self::Error> {
        self.write_tab()?;
        writeln!(&mut self.writer, "ptr += {};", offset)
    }

    fn add_set_value(&mut self, value: i8, offset: isize) -> Result<(), Self::Error> {
        self.write_tab()?;
        writeln!(&mut self.writer, "*(ptr + {}) = {};", offset, value)
    }

    fn add_inc_value(&mut self, inc: i8, offset: isize) -> Result<(), Self::Error> {
        self.write_tab()?;
        writeln!(&mut self.writer, "*(ptr + {}) += {};", offset, inc)
    }

    fn add_print(&mut self, offset: isize) -> Result<(), Self::Error> {
        self.write_tab()?;
        writeln!(&mut self.writer, "putchar(*(ptr + {}));", offset)
    }

    fn add_read(&mut self, offset: isize) -> Result<(), Self::Error> {
        self.write_tab()?;
        writeln!(&mut self.writer, "*(ptr + {}) = getchar();", offset)
    }

    fn add_multiply(&mut self, factor: i8, offset: isize) -> Result<(), Self::Error> {
        self.write_tab()?;
        writeln!(&mut self.writer, "*(ptr + {}) += *(ptr) * {};", offset, factor)
    }

    fn add_loop(&mut self, sub: &Vec<Atom>) -> Result<(), Self::Error> {
        self.write_tab()?;
        writeln!(&mut self.writer, "while(*ptr) {{")?;
        self.current_tab += 1;
        self.add_atoms(sub)?;
        self.current_tab -= 1;
        self.write_tab()?;
        writeln!(&mut self.writer, "}}")
    }
}
