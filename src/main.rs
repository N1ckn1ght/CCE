mod board;
mod engine;
mod characters;
mod utils;

use board::board::Board;
use characters::materialist::Materialist;
use engine::{character::Character, minimax::Minimax};
use std::{env, io::{stdin, stdout, Write}};
use crate::utils::utils::{move_to_user, move_to_board};

pub fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    test_loop("5k2/5ppp/5PPP/8/8/8/4R3/4R1K1 w - - 0 1", &Materialist{}, 2);
}

// some tests
pub fn test_loop<Char: Character>(FEN: &str, char: &Char, half_depth: i8) {
    let mut b = Board::parse_fen(FEN);
    let mut engine = Minimax::new();
    loop {
        b.print();
        let moves = engine.eval(&mut b, char, half_depth);
        println!("Total moves: {}", moves.len());
        for emov in &moves {
            println!("{}, score: {}, mate_in: {}", move_to_user(&b, &emov.mov), emov.eval.score, emov.eval.mate_in);
        }

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
                for emov in &moves {
                    if move_to_user(&b, &emov.mov) == command {
                        b.make_move(&move_to_board(&b, &command));
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