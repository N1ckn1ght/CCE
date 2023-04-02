use crate::board::coord::Coord;

#[derive(Clone, Copy, PartialEq)]
pub struct Mov {
    // data for keeping track on captures, promotions, special kind of moves, defined by Bimaps constants
    // also used in pre-ordering moves for faster a/b pruning
    pub data: u8,
    pub from: Coord,
    pub to: Coord
}

// the problem is: more additional info still needs to be stored in case of a move takeback!..
// it'd be also a bad idea to mash that info into data for a sort
#[derive(Clone)]
pub struct BoardMov {
    pub mov: Mov,
    pub castling: u8,
    pub en_passant: Coord,
    pub hmw: u8
}