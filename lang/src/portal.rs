use crate::Atom;

// TODO: Portal is currently unused. `Interpreter::execute` returns a bare
// Atom and the roadmap (issues 03-05) has Tick Plans subsume result destinations, so
// this module is retained pending that work rather than removed.

#[derive(Clone, Debug)]
pub struct Portal {
    pub atom: Atom,
    pub x: usize,
    pub y: usize,
}

impl Portal {
    pub fn new(atom: Atom, x: usize, y: usize) -> Self {
        Self { atom, x, y }
    }
}

impl Default for Portal {
    fn default() -> Self {
        Self {
            atom: Atom::Empty,
            x: 0,
            y: 1,
        }
    }
}
