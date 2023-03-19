use crate::board::coord::Coord;

#[derive(Clone)]
pub struct Mov {
    // data will store bits as follows:
    // 1 bit - is this move a double check? -- TODO
    // 2 bit - is this move a check?
    // 3 bits - 5 bits: is this move a capture?
    // 110 - queen, 101 - rook, 100 - bishop, 011 - knight, 001 - pawn
    // 6 bits - 7 bits: is this move a promotion?
    // 11 - queen, 10 - knight, 01 - rook, 00 - bishop
    // 8 bit - is this move a special move? (castling, en passant)
    pub data: u8,
    pub from: Coord,
    pub to: Coord
}