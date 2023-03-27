mod board;
use crate::board::{mov::Mov};

pub fn main() {
    use std::io::{stdin, stdout, Write};
    use crate::board::board::Board;

    // some tests
    let mut b = Board::parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string());

    b.print();
    let mut moves: Vec<Mov> = b.get_legal_moves(None);
    println!("Total moves: {}", moves.len());
    for i in 0..moves.len() {
        // println!("{} {} -> {} {}", moves[i].from.y, moves[i].from.x, moves[i].to.y, moves[i].to.x);
        println!("{}", move_from_board(moves[i]));
    }
}

pub fn move_to_board(umov: String) {
    
}

pub fn move_from_board(mov: Mov) -> String {
    let mut output = String::new();
    output.push((mov.from.x + 97) as char);
    output.push((mov.from.y + 49) as char);
    output.push((mov.to.x   + 97) as char);
    output.push((mov.to.y   + 49) as char);
    output
}