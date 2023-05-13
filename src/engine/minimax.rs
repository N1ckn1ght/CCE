use std::{cmp::{max, min, Ordering}};
use crate::{board::{board::{Board, Check}, mov::{Mov}}};
use super::{eval::{EvalMov, Eval}, character::Character};

// this will copy the first minimax iteration
// the purpose is to have a vector of evaluated possible moves as an output, not just the best one IF necessary
// initial alpha and beta are recommended to be just Eval::low(), Eval::high()
pub fn eval<Char: Character>(board: &mut Board, char: &mut Char, mut alpha: Eval, mut beta: Eval) -> Vec<EvalMov> {
    if char.get_static_half_depth() < 1 {
        panic!("0-half-depth minimax search attempt!\nStatic half depth must be at least 1, ideally divisible by 2.");
    }

    // set current position on board as played (draw-repetiion case)
    char.cache_play(char.make_hash(board));

    let mut moves: Vec<Mov> = board.get_legal_moves(Some(Check::Unknown), Some(true));
    let mut evals: Vec<EvalMov> = Vec::default();
    
    // pre-sort in descending order by Mov data (will fasten a/b pruning)
    moves.sort_by(|a, b| b.data.cmp(&a.data));

    for mov in &moves {
        board.make_move(mov);
        evals.push(EvalMov{ 
            mov: *mov, 
            eval: minimax(board, char, 1, alpha, beta, board.white_to_move, board.get_check(&mov.data)) });
        board.revert_move();

        if board.white_to_move {
            alpha = max(alpha, evals.last().unwrap().eval);
        } else {
            beta = min(beta, evals.last().unwrap().eval);
        }
        if beta <= alpha {
            break;
        }
    }

    // sort evaluated moves from the best to the worst in according to the current player to move
    if board.white_to_move {
        evals.sort_by(|a, b| b.eval.cmp(&a.eval));
    } else {
        evals.sort_by(|a, b| a.eval.cmp(&b.eval));
    }
    evals
}

// will return score eval and the mate_in moves if there's a forced checkmate sequence
fn minimax<Char: Character>(board: &mut Board, char: &mut Char, depth: i8, mut alpha: Eval, mut beta: Eval, maximize: bool, check: Check) -> Eval {
    let hash = char.make_hash(board);
    if char.is_played(hash) {
        return Eval::equal();
    }
    if char.is_evaluated(hash) {
        let stored_eval = char.get_hashed_eval(hash);
        let stored_depth = char.get_hashed_depth(hash);
        if depth < stored_depth {
            match stored_eval.mate_in.cmp(&0) {
                Ordering::Less => {
                    let eval = Eval::new(stored_eval.score, -depth);
                    char.cache_evaluated(hash, eval, depth, true);
                    return eval;
                },
                Ordering::Equal => {
                    char.cache_play(hash);
                },
                Ordering::Greater => {
                    let eval = Eval::new(stored_eval.score, depth);
                    char.cache_evaluated(hash, eval, depth, true);
                    return eval;
                },
            }
        } else {
            return stored_eval;
        }
    } else {
        char.cache_play(hash);
    }

    // It might be even faster to check for half_depth before, but it just feels wrong
    // get_legal_moves is a __relatively__ heavy method
    // the fact that there are certain moves may be useful to get interesting static results
    let mut eval: Eval;
    let mut moves: Vec<Mov> = board.get_legal_moves(Some(check), Some(true));

    if moves.is_empty() {
        let eval = match check {
            Check::InCheck | Check::InDoubleCheck => {
                let mate_in = if maximize {
                    // Black won
                    -depth
                } else {
                    // White won
                    depth
                };
                Eval {
                    score: char.get_static_eval_mate(board),
                    mate_in
                }
            },
            // Stalemate
            _ => Eval {
                score: char.get_static_eval_stalemate(board),
                mate_in: 0
            }
        };
        char.cache_unplay(hash);
        char.cache_evaluated(hash, eval, depth, false);
        return eval;
    }

    // pre-sort in descending order by Mov data (will fasten a/b pruning)
    moves.sort_by(|a, b| b.data.cmp(&a.data));

    if depth >= char.get_static_half_depth() {
        // quiescense search
        
        
        let eval = Eval { score: char.get_static_eval(board), mate_in: 0 };
        char.cache_unplay(hash);
        char.cache_evaluated(hash, eval, depth, false);
        return eval;
    }
    
    if maximize {
        eval = Eval::lowest();
        for mov in &moves {
            board.make_move(mov);
            let temp = minimax(board, char, depth + 1, alpha, beta, board.white_to_move, board.get_check(&mov.data));
            board.revert_move();
            eval = max(eval, temp);
            alpha = max(alpha, temp);
            if beta <= alpha {
                break;
            }
        }
    } else {
        eval = Eval::highest();
        for mov in &moves {
            board.make_move(mov);
            let temp = minimax(board, char, depth + 1, alpha, beta, board.white_to_move, board.get_check(&mov.data));
            board.revert_move();
            eval = min(eval, temp);
            beta = min(beta, temp);
            if beta <= alpha {
                break;
            }
        }
    }

    char.cache_unplay(hash);
    char.cache_evaluated(hash, eval, depth, false);
    eval
}


// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{characters::materialist::Materialist, utils::utils::move_to_user};

//     // Mate-in-X moves minimax engine test
//     // Default "Materialist" character will be used

//     #[test]
//     // Mate in 1, depth 3, linear;
//     fn test_minimax_find_mate_01() {
//         let mut b = Board::parse_fen(&"5k2/5ppp/5PPP/8/8/8/4R3/4R1K1 w - - 0 1");
//         let mut engine = Minimax::new();
//         let moves = engine.eval(&mut b, &Materialist{}, 6, Eval::lower(), Eval::higher(), true);
//         assert_eq!(move_to_user(&b, &moves[0].mov) == "e2e8", true);
//         assert_eq!(moves[0].eval.mate_in == 1, true);
//     }

//     #[test]
//     // Mate in 1, depth 1, promotion;
//     fn test_minimax_find_mate_02() {
//         let mut b = Board::parse_fen(&"6k1/4Pppp/5P2/8/8/8/8/6K1 w - - 0 1");
//         let mut engine = Minimax::new();
//         let moves = engine.eval(&mut b, &Materialist{}, 2, Eval::low(), Eval::high(), true);
//         assert_eq!(move_to_user(&b, &moves[0].mov) == "e7e8q" || move_to_user(&b, &moves[0].mov) == "e7e8r", true);
//         assert_eq!(moves[0].eval.mate_in == 1, true);
//     }

//     #[test]
//     // Mate in 2, depth 2, promotion;
//     fn test_minimax_find_mate_03() {
//         let mut b = Board::parse_fen(&"6kq/5ppp/4P3/8/8/8/8/BB4K1 w - - 0 1");
//         let mut engine = Minimax::new();
//         let moves = engine.eval(&mut b, &Materialist{}, 4, Eval::low(), Eval::high(), true);
//         assert_eq!(move_to_user(&b, &moves[0].mov) == "e6e7", true);
//         assert_eq!(moves[0].eval.mate_in == 2, true);
//     }

//     #[test]
//     // Mate in 2, depth 2, forced Legal Mate sequence;
//     fn test_minimax_find_mate_04() {
//         let mut b = Board::parse_fen(&"r2qkbnr/ppp2ppp/2np4/4N3/2B1P3/2N4P/PPPP1PP1/R1BbK2R w KQkq - 0 7");
//         let mut engine = Minimax::new();
//         let moves = engine.eval(&mut b, &Materialist{}, 4, Eval::low(), Eval::high(), true);
//         assert_eq!(move_to_user(&b, &moves[0].mov) == "c4f7", true);
//         assert_eq!(moves[0].eval.mate_in == 2, true);
//     }

//     #[test]
//     // Mate in -1, depth 1, almost Fool's mate;
//     fn test_minimax_find_mate_05() {
//         let mut b = Board::parse_fen(&"rnbqkbnr/pppp1ppp/8/8/4pPP1/P7/1PPPP2P/RNBQKBNR b KQkq f3 0 3");
//         let mut engine = Minimax::new();
//         let moves = engine.eval(&mut b, &Materialist{}, 2, Eval::low(), Eval::high(), true);
//         assert_eq!(move_to_user(&b, &moves[0].mov) == "d8h4", true);
//         assert_eq!(moves[0].eval.mate_in == -1, true);
//     }

//     #[test]
//     // Mate in -3, depth 3, linear mate, but White is to move and lose;
//     fn test_minimax_find_mate_06() {
//         let mut b = Board::parse_fen(&"k3r3/3r4/8/8/8/8/8/5K2 w - - 0 1");
//         let mut engine = Minimax::new();
//         let moves = engine.eval(&mut b, &Materialist{}, 6, Eval::new(0.0, -3), Eval::high(), true);
//         assert_eq!(moves[0].eval.mate_in == -3, true);
//     }

//     #[test]
//     // Mate in 2, depth 3, Vukovic Mate #3 (lichess.org), position from Pavol Danek - Stanislav Hanuliak, 2001;
//     fn test_minimax_find_mate_07() {
//         let mut b = Board::parse_fen(&"2r5/8/8/5K1k/4N1R1/7P/8/8 w - - 12 67");
//         let mut engine = Minimax::new();
//         let moves = engine.eval(&mut b, &Materialist{}, 6, Eval::low(), Eval::high(), true);
//         assert_eq!(move_to_user(&b, &moves[0].mov) == "e4f6", true);
//         assert_eq!(moves[0].eval.mate_in == 2, true);
//     }

//     #[test]
//     // Mate in -2, depth 3, promotion with capture
//     fn test_minimax_find_mate_08() {
//         let mut b = Board::parse_fen(&"8/6N1/b7/8/6k1/3Q4/2pp1PPP/4B1K1 b - - 0 1");
//         let mut engine = Minimax::new();
//         let moves = engine.eval(&mut b, &Materialist{}, 6, Eval::low(), Eval::high(), true);
//         assert_eq!(move_to_user(&b, &moves[0].mov) == "d2e1q" || move_to_user(&b, &moves[0].mov) == "d2e1r", true);
//         assert_eq!(moves[0].eval.mate_in == -2, true);
//     }

//     #[test]
//     // Mate in 1, depth 2 long castle
//     fn test_minimax_find_mate_09() {
//         let mut b = Board::parse_fen(&"r2k1bnr/ppp1pppp/5N2/8/8/7B/PPP1PP1P/R3K1NR w KQ - 0 1");
//         let mut engine = Minimax::new();
//         let moves = engine.eval(&mut b, &Materialist{}, 4, Eval::low(), Eval::high(), true);
//         assert_eq!(move_to_user(&b, &moves[0].mov) == "e1c1", true);
//         assert_eq!(moves[0].eval.mate_in == 1, true);
//     }

//     #[test]
//     // Mate in -2, depth 2, short castle or Rf8
//     fn test_minimax_find_mate_10() {
//         let mut b = Board::parse_fen(&"rnb1k2r/pppp2pp/8/2b5/7Q/8/PPPPP1PP/1NBQRKR1 b kq - 0 1");
//         let mut engine = Minimax::new();
//         let moves = engine.eval(&mut b, &Materialist{}, 4, Eval::low(), Eval::high(), true);
//         assert_eq!(move_to_user(&b, &moves[0].mov) == "e8g8" || move_to_user(&b, &moves[0].mov) == "h8f8", true);
//         assert_eq!(moves[0].eval.mate_in == -2, true);
//     }

//     #[test]
//     // Mate in -1, depth 2, symmetrical, smoothered mate, board is in chaos
//     fn test_minimax_find_mate_11() {
//         let mut b = Board::parse_fen(&"qqq3rk/bbnP2pp/qqq2p2/4p1N1/4P1n1/QQQ2P2/BBNp2PP/QQQ3RK w - - 0 1");
//         let mut engine = Minimax::new();
//         let moves = engine.eval(&mut b, &Materialist{}, 4, Eval::low(), Eval::high(), true);
//         assert_eq!(move_to_user(&b, &moves[0].mov) == "g5f7", true);
//         assert_eq!(moves[0].eval.mate_in == 1, true);
//     }
    
//     #[test]
//     // Mate in -1, depth 3, almost Fool's mate;
//     fn test_minimax_find_long_mate_01() {
//         let mut b = Board::parse_fen(&"rnbqkbnr/pppp1ppp/8/8/4pPP1/P7/1PPPP2P/RNBQKBNR b KQkq f3 0 3");
//         let mut engine = Minimax::new();
//         let moves = engine.eval(&mut b, &Materialist{}, 6, Eval::low(), Eval::high(), true);
//         assert_eq!(move_to_user(&b, &moves[0].mov) == "d8h4", true);
//         assert_eq!(moves[0].eval.mate_in == -1, true);
//     }

//     #[test]
//     // Mate in 4, depth 4, first move is a castle;
//     fn test_minimax_find_long_mate_02() {
//         let mut b = Board::parse_fen(&"r1bk1bnr/ppp1pppp/5N2/8/8/7B/PPP1PP1P/R3K1NR w KQ - 0 1");
//         let mut engine = Minimax::new();
//         let moves = engine.eval(&mut b, &Materialist{}, 8, Eval::low(), Eval::high(), true);
//         assert_eq!(move_to_user(&b, &moves[0].mov) == "e1c1", true);
//         assert_eq!(moves[0].eval.mate_in == 4, true);
//     }

//     #[test]
//     // Mate in 5, depth 4, use hashing, mate is kinda forced;
//     fn test_minimax_find_hash_mate_01() {
//         let mut b = Board::parse_fen(&"2q1nk1r/4Rp2/1ppp1P2/6Pp/3p1B2/3P3P/PPP1Q3/6K1 w - - 0 1");
//         let mut engine = Minimax::new();
//         let moves = engine.eval(&mut b, &Materialist{}, 8, Eval::low(), Eval::high(), true);
//         assert_eq!(move_to_user(&b, &moves[0].mov) == "e7e8", true);
//         assert_eq!(moves[0].eval.mate_in == 5, true);
//     }

//     // Long tests that can be ignored, unless it is a perfomance test.

//     #[test]
//     #[ignore]
//     // Mate in 3, depth 3, bishop and queen traps the castled king;
//     fn test_minimax_find_quiet_mate_01() {
//         let mut b = Board::parse_fen(&"4qrk1/p1r1Bppp/4b3/2p3Q1/8/3P4/PPP2PPP/R3R1K1 w - - 3 19");
//         let mut engine = Minimax::new();
//         let moves = engine.eval(&mut b, &Materialist{}, 6, Eval::low(), Eval::high(), true);
//         assert_eq!(move_to_user(&b, &moves[0].mov) == "e7f6", true);
//         assert_eq!(moves[0].eval.mate_in == 3, true);
//     }

//     #[test]
//     #[ignore]
//     // Mate in 5, depth 4, use hashing, mate is kinda forced;
//     fn test_minimax_find_long_mate_03() {
//         let mut b = Board::parse_fen(&"6k1/3b3r/1p1p4/p1n2p2/1PPNpP1q/P3Q1p1/1R1RB1P1/5K2 b - - 0 1");
//         let mut engine = Minimax::new();
//         let moves = engine.eval(&mut b, &Materialist{}, 10, Eval::low(), Eval::high(), true);
//         assert_eq!(move_to_user(&b, &moves[0].mov) == "h4f4", true);
//         assert_eq!(moves[0].eval.mate_in == -5, true);
//     }
// }