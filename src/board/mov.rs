use crate::board::coord::Coord;

#[derive(Clone)]
pub struct Mov {
    // data will store bits as follows:
    // 1 bit - is this move a mate?
    // 2 bit - is this move a check?
    // 3 bit - is this move a capture?
    // 4 bit - is this move special? (castling, en passant)
    // next 3 bits will define a piece that was captured, if there was a capture:
    // 110 - queen, 101 - rook, 100 - bishop, 011 - knight, 001 - pawn
    // last bit determines the color of the captured piece, it doesn't contribute to sort
    pub data: u8,
    pub from: Coord,
    pub to: Coord
}