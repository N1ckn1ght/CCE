mod board;

use std::{cmp::{max, min}, io::{stdin, stdout, Write}};
use crate::board::{board::Board, coord::Coord, mov::Mov};
use std::env;

pub fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    test_loop(None);
}

// some tests
pub fn test_loop(FEN: Option<String>) {
    let mut b = Board::parse_fen(FEN.unwrap_or("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string()));

    loop {
        b.print();
        let mut moves: Vec<Mov> = b.get_legal_moves(None);
        println!("Total moves: {}", moves.len());
        for i in 0..moves.len() {
            print!("{} ", move_to_user(&b, &moves[i]));
        }
        println!();

        let mut success: bool = false;
        while !success {
            let mut command = String::new();
            stdin().read_line(&mut command).expect("Input fail");
            command = command.trim().to_string();
            let _ = stdout().flush();

            if command == "exit" {
                return;
            } else if command == "takeback" {
                b.revert_move();
                success = true;
            } else {
                for i in 0..moves.len() {
                    if move_to_board(&b, &command) == moves[i] {
                        b.make_move(move_to_board(&b, &command));
                        success = true;
                        break;
                    }
                }
            }

            if !success {
                println!("Invalid command? Bug in the program? That's sad :(");
            }
        }
    }
}

// warning: this function works assuming that the usermove is already legit!
pub fn move_to_board(board: &Board, umov: &String) -> Mov {
    let chars: Vec<char> = umov.chars().collect();
    let from     = Coord::new(chars[1] as u8 - 49, chars[0] as u8 - 97);
    let to       = Coord::new(chars[3] as u8 - 49, chars[2] as u8 - 97);
    let mut data = 0;

    let color_bit = board.white_to_move as u8;
    // promotion handling
    if chars.len() > 4 {
        data |= board.grls(&chars[4]) | 1;
    }
    // castle handling (TODO: accept fischer castlings)
    else if board.field[from.y() as usize][from.x() as usize] == board.gpl(&'k') + color_bit as u8 && max(from.x(), to.x()) - min(from.x(), to.x()) > 1 {
        data |= 1;
    }
    // en passant handling
    else if board.field[from.y() as usize][from.x() as usize] == board.gpl(&'p') + color_bit as u8 && board.en_passant.y() == to.y() && board.en_passant.x() == to.x() {
        data |= 1;
        // add pawn capture as well
        data |= board.gpls(&'p');
    }
    // capture handling (won't proc in case of en passant because it's impossible to have a piece on that square)
    data |= board.psav(board.field[to.y() as usize][to.x() as usize]);
    // no check handlings for now
    // println!("debug, move made, data: {}, from: {}, {}, to: {}, {}", data, from.y(), from.x(), to.y(), to.x());
    Mov{data, from, to}
}

pub fn move_to_user(board: &Board, mov: &Mov) -> String {
    let mut output = String::new();
    output.push((mov.from.x() + 97) as char);
    output.push((mov.from.y() + 49) as char);
    output.push((mov.to.x()   + 97) as char);
    output.push((mov.to.y()   + 49) as char);
    if board.is_promotion(mov) {
        output.push(board.rtpc(mov.data));
    }
    output
}