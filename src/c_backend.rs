use std::io::{self, Write};

use ir::Atom;

const MEM_SIZE: usize = 30000;

pub fn write_from_ir<W: Write>(ir: &Vec<Atom>, writer: W) -> io::Result<()> {
    let mut backend = CBackend::new(writer);
    backend.generate_header()?;
    backend.generate(ir)?;
    backend.generate_footer()?;
    Ok(())
}

#[derive(Debug, Clone)]
struct CBackend<W: Write> {
    writer: W,
    current_tab: usize,
}

impl<W: Write> CBackend<W> {
    fn new(writer: W) -> Self {
        CBackend {
            writer,
            current_tab: 1,
        }
    }

    fn generate_tab(&mut self) -> io::Result<()> {
        write!(&mut self.writer, "{}", "\t".repeat(self.current_tab))
    }

    fn generate_header(&mut self) -> io::Result<()> {
        writeln!(&mut self.writer, "#include <stdlib.h>")?;
        writeln!(&mut self.writer, "#include <stdio.h>")?;
        writeln!(&mut self.writer, "#include <stdint.h>")?;

        writeln!(&mut self.writer, "int8_t memory[{}];", MEM_SIZE)?;
        writeln!(&mut self.writer, "int8_t* ptr = memory;")?;
        writeln!(&mut self.writer, "int main() {{")?;
        Ok(())
    }

    fn generate_footer(&mut self) -> io::Result<()> {
        writeln!(&mut self.writer, "}}")
    }
    
    fn generate(&mut self, ir: &Vec<Atom>) -> io::Result<()> {
        for atom in ir {
            self.generate_tab()?;
            match *atom {
                Atom::MovePtr(offset)
                    => writeln!(&mut self.writer, "ptr += {};", offset)?,
                Atom::SetValue(value, offset)
                    => writeln!(&mut self.writer, "*(ptr + {}) = {};", offset, value)?,
                Atom::IncValue(inc, offset)
                    => writeln!(&mut self.writer, "*(ptr + {}) += {};", offset, inc)?,
                Atom::Print(offset)
                    => writeln!(&mut self.writer, "putchar(*(ptr + {}));", offset)?,
                Atom::Read(offset)
                    => writeln!(&mut self.writer, "*(ptr + {}) = getchar();", offset)?,
                Atom::Multiply(factor, offset)
                    => writeln!(&mut self.writer, "*(ptr + {}) += *(ptr) * {};", offset, factor)?,
                Atom::Loop(ref sub) => {
                    writeln!(&mut self.writer, "while(*ptr) {{")?;
                    self.current_tab += 1;
                    self.generate(sub)?;
                    self.current_tab -= 1;
                    self.generate_tab()?;
                    writeln!(&mut self.writer, "}}")?;
                }
            }
        }
        Ok(())
    }
}
