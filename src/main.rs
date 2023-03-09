mod board;

pub fn main() {
    use std::io::{stdin, stdout, Write};
    use crate::board::board::Board;

    // some tests
    let mut b = Board::parse_fen("rnbqkbnr/pppppp1p/6p1/4P3/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 2".to_string());
    let mut c = Board::parse_fen("rnbq1bnr/ppppkppp/8/4p3/4P3/8/PPPPKPPP/RNBQ1BNR w - - 2 3".to_string());
    let mut d = Board::clone(&c);
    c.field[0][0] = 0;

    b.print();
    c.print();
    d.print();
}