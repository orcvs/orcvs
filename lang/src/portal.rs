use crate::Atom;

// TODO: Portal is currently unused. Tick Plans now own result destinations;
// this legacy type is retained pending the remaining Source playback work.

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
