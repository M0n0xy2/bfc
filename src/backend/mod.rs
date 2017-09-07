use ir::Atom;

pub mod c;
pub mod interpreter;
pub use self::c::CBackend;
pub use self::interpreter::Interpreter;

pub fn use_backend<B: Backend>(mut backend: B, ir: &Vec<Atom>)
    -> Result<B::Payload, B::Error> {
    backend.initialize()?;
    backend.push_atoms(ir)?;
    backend.finalize()
}

pub trait Backend {
    type Payload;
    type Error;

    fn initialize(&mut self) -> Result<(), Self::Error>;
    fn finalize(self) -> Result<Self::Payload, Self::Error>;

    fn push_atoms(&mut self, ir: &Vec<Atom>) -> Result<(), Self::Error> {
        for atom in ir {
            self.push_atom(atom)?;
        }
        Ok(())
    }
    fn push_atom(&mut self, atom: &Atom) -> Result<(), Self::Error> {
        match *atom {
            Atom::MovePtr(offset) => self.push_move_ptr(offset),
            Atom::SetValue(value, offset) => self.push_set_value(value, offset),
            Atom::IncValue(inc, offset) => self.push_inc_value(inc, offset),
            Atom::Print(offset) => self.push_print(offset),
            Atom::Read(offset) => self.push_read(offset),
            Atom::Multiply(factor, offset) => self.push_multiply(factor, offset),
            Atom::Loop(ref sub) => self.push_loop(sub),
        }
    }

    fn push_move_ptr(&mut self, offset: isize) -> Result<(), Self::Error>;
    fn push_set_value(&mut self, value: i8, offset: isize) -> Result<(), Self::Error>;
    fn push_inc_value(&mut self, inc: i8, offset: isize) -> Result<(), Self::Error>;
    fn push_print(&mut self, offset: isize) -> Result<(), Self::Error>;
    fn push_read(&mut self, offset: isize) -> Result<(), Self::Error>;
    fn push_multiply(&mut self, factor: i8, offset: isize) -> Result<(), Self::Error>;
    fn push_loop(&mut self, sub: &Vec<Atom>) -> Result<(), Self::Error>;
}
