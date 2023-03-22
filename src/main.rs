mod board;

pub fn main() {
    use std::io::{stdin, stdout, Write};
    use crate::board::board::Board;

    // some tests
    let mut b = Board::parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string());

    
}