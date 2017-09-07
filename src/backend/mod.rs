use ir::Atom;

pub mod c;
pub use self::c::CBackend;

pub fn use_backend<B: Backend>(mut backend: B, ir: &Vec<Atom>)
    -> Result<B::Payload, B::Error> {
    backend.initialize()?;
    backend.add_atoms(ir)?;
    backend.finalize()
}

pub trait Backend {
    type Payload;
    type Error;

    fn initialize(&mut self) -> Result<(), Self::Error>;
    fn finalize(self) -> Result<Self::Payload, Self::Error>;

    fn add_atoms(&mut self, ir: &Vec<Atom>) -> Result<(), Self::Error> {
        for atom in ir {
            self.add_atom(atom)?;
        }
        Ok(())
    }
    fn add_atom(&mut self, atom: &Atom) -> Result<(), Self::Error> {
        match *atom {
            Atom::MovePtr(offset) => self.add_move_ptr(offset),
            Atom::SetValue(value, offset) => self.add_set_value(value, offset),
            Atom::IncValue(inc, offset) => self.add_inc_value(inc, offset),
            Atom::Print(offset) => self.add_print(offset),
            Atom::Read(offset) => self.add_read(offset),
            Atom::Multiply(factor, offset) => self.add_multiply(factor, offset),
            Atom::Loop(ref sub) => self.add_loop(sub),
        }
    }

    fn add_move_ptr(&mut self, offset: isize) -> Result<(), Self::Error>;
    fn add_set_value(&mut self, value: i8, offset: isize) -> Result<(), Self::Error>;
    fn add_inc_value(&mut self, inc: i8, offset: isize) -> Result<(), Self::Error>;
    fn add_print(&mut self, offset: isize) -> Result<(), Self::Error>;
    fn add_read(&mut self, offset: isize) -> Result<(), Self::Error>;
    fn add_multiply(&mut self, factor: i8, offset: isize) -> Result<(), Self::Error>;
    fn add_loop(&mut self, sub: &Vec<Atom>) -> Result<(), Self::Error>;
}
