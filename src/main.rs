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
    
    // test_loop("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", &Materialist{}, 4);
    test_loop("r2qkbnr/ppp2ppp/2np4/4N3/2B1P3/2N4P/PPPP1PP1/R1BbK2R w KQkq - 0 7", &Materialist{}, 4);
    // test_loop("6k1/3b3r/1p1p4/p1n2p2/1PPNpP1q/P3Q1p1/1R1RB1P1/5K2 b - - 0 1", &Materialist{}, 10);
    
    // test_loop("2q1nk1r/4Rp2/1ppp1P2/6Pp/3p1B2/3P3P/PPP1Q3/6K1 w - - 0 1", &Materialist{}, 10);
    // test_loop("4qrk1/p1r1Bppp/4b3/2p3Q1/8/3P4/PPP2PPP/R3R1K1 w - - 3 19", &Materialist{}, 6);
}

// some tests
pub fn test_loop<Char: Character>(FEN: &str, char: &Char, mut half_depth: i8) {
    let mut b = Board::parse_fen(FEN);
    let mut engine = Minimax::new();
    let mut any_mate = true;
    let mut prune_first = true;
    loop {
        println!("\n--------------------------------------------------\n");
        b.print();
        let moves = engine.eval(&mut b, char, half_depth, any_mate, prune_first);
        println!("Total moves: {}", moves.len());
        for emov in &moves {
            println!("{}, score: {}, mate_in: {}", move_to_user(&b, &emov.mov), emov.eval.score, emov.eval.mate_in);
        }
        println!();

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
            } else if command == "mode prune" {
                prune_first = !prune_first;
                println!("prune_first is now set to: {}", prune_first);
                opt = true;
            } else if command == "mode mate" {
                any_mate = !any_mate;
                println!("any_mate is now set to: {}", any_mate);
                opt = true;
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
                for emov in &moves {
                    if move_to_user(&b, &emov.mov) == command {
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