mod board;
mod engine;
mod characters;
mod utils;

use board::board::Board;

use characters::generic::Generic;
use engine::{character::Character};
use std::{env, io::{stdin, stdout, Write}};
use crate::{utils::utils::{move_to_user, move_to_board}};

pub fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    test_loop("k3r3/3r4/8/8/8/8/8/5K2 w - - 0 1", &mut Generic::new(), 6);
}

// some tests
pub fn test_loop<Char: Character>(FEN: &str, char: &mut Char, mut half_depth: i8) {
    let mut board = Board::parse_fen(FEN);
    char.set_static_half_depth(half_depth);
    loop {
        println!("\n--------------------------------------------------\n");
        board.print();
        let moves = char.get_eval_moves(&mut board);
        println!("Total moves: {}", moves.len());
        for emov in &moves {
            println!("{}, score: {}, mate_in: {}", move_to_user(&board, &emov.mov), emov.eval.score, emov.eval.mate_in);
        }
        println!();

        let legals = board.get_legal_moves(None, None);
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
                board.revert_move();
                success = true;
            } else if command == "depth up" {
                half_depth += 2;
                char.set_static_half_depth(half_depth);
                println!("depth is now set to: {}", half_depth >> 1);
                opt = true;
            } else if command == "depth down" {
                half_depth -= 2;
                char.set_static_half_depth(half_depth);
                println!("depth is now set to: {}", half_depth >> 1);
                opt = true;
            } else if command == "rethink" {
                success = true;
            } else {
                for lmov in &legals {
                    if move_to_user(&board, lmov) == command {
                        board.make_move(&move_to_board(&board, &command));
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