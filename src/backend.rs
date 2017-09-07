use ir::Atom;

pub trait Backend {
    type Payload;

    fn initialize();
    fn finalize() -> Payload;

    fn add_atoms(&mut self, ir: &Vec<Atom>) {
        for atom in ir {
            self.add_atom(atom);
        }
    }
    fn add_atom(&mut self, atom: &Atom) {
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

    fn add_move_ptr(&mut self, offset: isize);
    fn add_set_value(&mut self, value: i8, offset: isize);
    fn add_inc_value(&mut self, inv: i8, offset: isize);
    fn add_print(&mut self, offset: isize);
    fn add_read(&mut self, offset: isize);
    fn add_multiply(&mut self, factor: i8, offset: isize);
    fn add_loop(&mut self, sub: &Vec<Atom>);
}
