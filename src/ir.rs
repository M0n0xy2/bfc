#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Atom {
    MovePtr(isize),
    SetValue(u8, isize),
    IncValue(i8, isize),
    Print(isize),
    Read(isize),
    Multiply(i8, isize), // factor, offset
    Loop(Vec<Atom>),
}

#[derive(Debug, Clone)]
pub enum ParenError {
    RightMissing(usize),
    LeftMissing(usize),
}

struct IRBuilder {
    ir: Vec<Atom>,
    loops: Vec<(usize, Vec<Atom>)>,
}

impl IRBuilder {
    fn new() -> Self {
        IRBuilder {
            ir: Vec::new(),
            loops: Vec::new(),
        }
    }

    fn push_atom(&mut self, atom: Atom) {
        if let Some(&mut (_, ref mut current_loop)) = self.loops.last_mut() {
            current_loop.push(atom);
        } else {
            self.ir.push(atom);
        }
    }

    fn start_loop(&mut self, pos: usize) {
        self.loops.push((pos, Vec::new()));
    }

    fn end_loop(&mut self, pos: usize) -> Result<(), ParenError> {
        if let Some((_, last_loop)) = self.loops.pop() {
            self.push_atom(Atom::Loop(last_loop));
            Ok(())
        } else {
            Err(ParenError::LeftMissing(pos))
        }
    }

    fn collect(self) -> Result<Vec<Atom>, ParenError> {
        if let Some(&(pos, _)) = self.loops.first() {
            Err(ParenError::RightMissing(pos))
        } else {
            Ok(self.ir)
        }
    }
}

pub fn build_ir(input: &[u8]) -> Result<Vec<Atom>, ParenError> {
    let mut ir_builder = IRBuilder::new();

    for (pos, c) in input.into_iter().enumerate() {
        match *c {
            b'+' => ir_builder.push_atom(Atom::IncValue(1, 0)),
            b'-' => ir_builder.push_atom(Atom::IncValue(-1, 0)),
            b'<' => ir_builder.push_atom(Atom::MovePtr(-1)),
            b'>' => ir_builder.push_atom(Atom::MovePtr(1)),
            b'.' => ir_builder.push_atom(Atom::Print(0)),
            b',' => ir_builder.push_atom(Atom::Read(0)),
            b'[' => ir_builder.start_loop(pos),
            b']' => ir_builder.end_loop(pos)?,
            _ => {}
        }
    } 
    ir_builder.collect()
}
