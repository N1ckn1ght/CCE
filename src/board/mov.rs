use crate::board::coord::Coord;

#[derive(Clone)]
pub struct Mov {
    // data will store bits as follows:
    // 1 bit - is this move a double check? -- TODO
    // 2 bit - is this move a check?
    // 3 bit - is this move a capture?
    // 4 bit - is this move an any kind of a special move? (castling, en passant, promotion)
    // 5-7 bits will define a piece that was captured, if there was a capture:
    // 110 - queen, 101 - rook, 100 - bishop, 011 - knight, 001 - pawn
    // 8 bit determines the color of the captured piece, it doesn't contribute to sort
    //
    // hotfix change: in case of promotion in order to keep capture x promotion check 5-8 bits will store data as follows:
    // 5-6 bits will determine promotion (11 for q, 10 for n, 01 for r, 00 for b)
    // 7-8 bits will determine capture   (same, keep in mind to look for 3 bit if there ever was a capture)
    // it's not possible to promote to a pawn or to capture a pawn at the promotion, but only in the basic chess ruleset
    // TODO - make data storage the way so custom impossible positions would be allowed, such as pawns at rank 8
    pub data: u8,
    pub from: Coord,
    pub to: Coord
}