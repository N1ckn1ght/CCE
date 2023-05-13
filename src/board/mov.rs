use super::{board::Board, coord::Coord};

#[derive(Clone, Copy, PartialEq)]
pub struct Mov {
    // data for keeping track on captures, promotions, special kind of moves, defined by Bimaps constants
    // also used in pre-ordering moves for faster a/b pruning
    pub data: u8,
    pub from: Coord,
    pub to: Coord
}

impl Mov {
    // return true if this move is a check or a capture
    // look bimaps for the reference
    pub fn is_dynamic(&self) -> bool {
        self.data > 7
    }

    // return true if this move is NOT a capture or a pawn move
    // does require a board as a context
    pub fn is_repeatable(&self, board: &Board) -> bool {
        let mov = board.history.first().unwrap().mov;
        board.field[mov.to.y() as usize][mov.to.x() as usize] & 254 == board.gpl(&'p') || board.ptpv(mov.data) > 1
    }
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