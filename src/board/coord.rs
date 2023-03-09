#[derive(Clone)]
pub struct Coord {
    pub y: u8,
    pub x: u8
}

impl Coord {
    pub fn new(y: u8, x: u8) -> Coord {
        Coord{y, x}
    }
}