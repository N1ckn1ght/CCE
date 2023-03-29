#[derive(Clone, Copy, PartialEq)]
pub struct Coord {
    value: u8
}

impl Coord {
    pub fn new(y: u8, x: u8) -> Coord {
        let value = (y << 4) + (x & 15);
        Coord{value}
    }

    pub fn y(& self) -> u8 {
        self.value >> 4
    }

    pub fn x(& self) -> u8 {
        self.value & 15
    }

    pub fn set(&mut self, y: u8, x: u8) {
        self.value = (y << 4) + (x & 15);
    }
}