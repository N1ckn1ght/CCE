use std::vec;

use crate::board::{mov::Mov, board::Check};

mod board;

pub fn main() {
    use std::io::{stdin, stdout, Write};
    use crate::board::board::Board;

    // some tests
    let mut b = Board::parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string());

    b.print();
    let mut moves: Vec<Mov> = b.get_legal_moves(None);
    for i in 0..moves.len() {
        println!("{} {} -> {} {}", moves[i].from.y, moves[i].from.x, moves[i].to.y, moves[i].to.x);
    }
}