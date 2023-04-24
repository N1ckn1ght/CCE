mod board;
mod engine;
mod characters;
mod utils;

use board::board::Board;
use characters::materialist::Materialist;
use engine::{character::Character, minimax::Minimax};
use std::{env, io::{stdin, stdout, Write}};
use crate::{utils::utils::{move_to_user, move_to_board}, engine::eval::Eval};

pub fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    test_loop("k3r3/3r4/8/8/8/8/8/5K2 w - - 0 1", &Materialist{}, 6);
}

// some tests
pub fn test_loop<Char: Character>(FEN: &str, char: &Char, mut half_depth: i8) {
    let mut b = Board::parse_fen(FEN);
    let mut engine = Minimax::new();
    loop {
        println!("\n--------------------------------------------------\n");
        b.print();
        let moves = engine.eval(&mut b, char, half_depth, Eval::low(), Eval::high(), false);
        println!("Total moves: {}", moves.len());
        for emov in &moves {
            println!("{}, score: {}, mate_in: {}", move_to_user(&b, &emov.mov), emov.eval.score, emov.eval.mate_in);
        }
        println!();

        let legals = b.get_legal_moves(None, None);
        let mut success: bool = false;
        while !success {
            let mut command = String::new();
            stdin().read_line(&mut command).expect("Input fail");
                command = command.trim().to_string();
            let _ = stdout().flush();

            let mut opt = false;
            if command == "exit" {
                return;
            } else if command == "takeback" {
                b.revert_move();
                success = true;
            } else if command == "depth up" {
                half_depth += 2;
                println!("depth is now set to: {}", half_depth >> 1);
                opt = true;
            } else if command == "depth down" {
                half_depth -= 2;
                println!("depth is now set to: {}", half_depth >> 1);
                opt = true;
            } else if command == "rethink" {
                success = true;
            } else {
                for lmov in &legals {
                    if move_to_user(&b, lmov) == command {
                        b.make_move(&move_to_board(&b, &command));
                        success = true;
                        break;
                    }
                }
            }

            if !(success || opt) {
                println!("Invalid command? Bug in the program? That's sad :(");
            }
        }
    }
}