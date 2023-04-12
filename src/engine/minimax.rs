use std::{cmp::{max, min, Ordering}, collections::HashMap};
use crate::{board::{board::{Board, Check}, mov::{Mov}}};
use super::{eval::{EvalMov, Eval}, character::Character, hashtable::Hashtable};

pub struct Minimax {
    called_half_depth: i8,
    hashtable: Hashtable,
    hashes: HashMap<u64, EvalNo>
}

impl Minimax {
    pub fn new() -> Self {
        Self { called_half_depth: 0, hashtable: Hashtable::new(1204), hashes: HashMap::new() }
    }

    // this will copy the first minimax iteration
    // the purpose is to have a vector of evaluated possible moves as an output, not just the best one
    pub fn eval<Char: Character>(&mut self, board: &mut Board, char: &Char, half_depth: i8, any_mate: bool, prune_first:bool) -> Vec<EvalMov> {
        // TODO: don't delete calculated positions like that!
        //       make use of them. this is just for the test.
        self.hashes.clear();

        if half_depth < 1 {
            panic!("0-half-depth evaluation attempt");
        }
        
        self.called_half_depth = half_depth;
        let mut moves: Vec<Mov> = board.get_legal_moves(Some(Check::Unknown), Some(true));
        let mut evals: Vec<EvalMov> = Vec::default();
        
        // pre-sort in descending order by Mov data (will fasten a/b pruning)
        // TODO: No, it will not! Because engine tries to evaluate every possible move
        //       do something with it, really.
        //       At lest in minimax() method this pre-sort works as intended.
        moves.sort_by(|a, b| b.data.cmp(&a.data));

        let mut alpha: Eval;
        let mut beta: Eval;

        // with any_mate there'll be a faster search for any mate in given position
        // takeback, however, is that found mate might not be the fastest possible
        if any_mate {
            alpha = Eval::low();
            beta = Eval::high();
        } else {
            alpha = Eval::lowest();
            beta = Eval::highest();
        }

        for mov in &moves {
            board.make_move(mov);
            evals.push(EvalMov{ 
                mov: *mov, 
                eval: self.minimax(board, char, half_depth - 1, alpha, beta, board.white_to_move, board.get_check(&mov.data)) });
            board.revert_move();

            // Just for debug by now, will be implmented properly later
            if prune_first {
                if board.white_to_move {
                    alpha = max(alpha, evals.last().unwrap().eval);
                } else {
                    beta = min(beta, evals.last().unwrap().eval);
                }
                if beta <= alpha {
                    break;
                }
            }
        }

        // transfer half moves to moves
        for em in &mut evals {
            em.eval.mate_in = em.eval.mate_in.signum() *  ((em.eval.mate_in.abs() + 1) >> 1);
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
    pub fn minimax<Char: Character>(&mut self, board: &mut Board, char: &Char, half_depth: i8, mut alpha: Eval, mut beta: Eval, maximize: bool, check: Check) -> Eval {
        
        // Work on handling same position is still in progress...
        let hash = self.hashtable.hash(board);
        let mut insert = true;
        if let Some(evaluated) = self.hashes.get_mut(&hash) {
            insert = false;
            let delta = self.called_half_depth - half_depth - evaluated.depth;
            match delta.cmp(&0){
                // This means we found a more quick way to get exactly this position!
                Ordering::Less => {
                    match evaluated.eval.mate_in.cmp(&0) {
                        Ordering::Less => {
                            evaluated.eval.mate_in -= delta;
                            return evaluated.eval;
                        },
                        Ordering::Equal => (), // Just calculate in more depth, there could be a mate in something
                                               // or just a better or a worse position than expected
                        Ordering::Greater => {
                            evaluated.eval.mate_in += delta;
                            return evaluated.eval;
                        }
                    }
                },
                // This is an already calculated transposition.
                Ordering::Equal => {
                    return evaluated.eval;
                },
                // We got there by making some reversible moves in-between.
                Ordering::Greater => {
                    match evaluated.eval.mate_in.cmp(&0) {
                        Ordering::Less => {
                            return Eval { score: evaluated.eval.score, mate_in: evaluated.eval.mate_in - delta };
                        },
                        Ordering::Equal => {
                            // TODO: analyze if it's a draw or a repetitive move
                            return evaluated.eval;
                        }
                        Ordering::Greater => {
                            return Eval { score: evaluated.eval.score, mate_in: evaluated.eval.mate_in + delta };
                        }
                    }
                    
                }
            }
        }

        // It might be even faster to check for half_depth before, but it just feels wrong
        // get_legal_moves is a relatively heavy method by now (which it shouldn't be however)
        // the fact that there are certain moves may be useful to get interesting static results
        let mut eval: Eval;
        let mut moves: Vec<Mov> = board.get_legal_moves(Some(check), Some(true));

        if moves.is_empty() {
            let eval = match check {
                Check::InCheck | Check::InDoubleCheck => {
                    let mate_in = if maximize {
                        // White won
                        half_depth - self.called_half_depth
                    } else {
                        // Black won
                        self.called_half_depth - half_depth
                    };
                    Eval {
                        score: char.static_eval_mate(board),
                        mate_in,
                    }
                },
                // Stalemate
                _ => Eval {
                    score: 0.0,
                    mate_in: 0,
                }
            };
            self.hashes.insert(hash, EvalNo { eval, depth: self.called_half_depth - half_depth });
            return eval;
        }

        // pre-sort in descending order by Mov data (will fasten a/b pruning)
        moves.sort_by(|a, b| b.data.cmp(&a.data));

        if half_depth < 1 {
            // TODO: run quiescense search first!
            let eval = Eval { score: char.static_eval(board), mate_in: 0 };
            self.hashes.insert(hash, EvalNo { eval, depth: self.called_half_depth });
            return eval;
        }
        
        if maximize {
            eval = Eval::lowest();
            for mov in &moves {
                board.make_move(mov);
                let temp = self.minimax(board, char, half_depth - 1, alpha, beta, board.white_to_move, board.get_check(&mov.data));
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
                let temp = self.minimax(board, char, half_depth - 1, alpha, beta, board.white_to_move, board.get_check(&mov.data));
                board.revert_move();
                eval = min(eval, temp);
                beta = min(beta, temp);
                if beta <= alpha {
                    break;
                }
            }
        }

        if insert {
            self.hashes.insert(hash, EvalNo { eval, depth: self.called_half_depth - half_depth });
        }
        eval
    }
}

pub struct EvalNo {
    pub eval: Eval,
    pub depth: i8
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{characters::materialist::Materialist, utils::utils::move_to_user};

    // Mate-in-X moves minimax engine test
    // Default "Materialist" character will be used

    #[test]
    // Mate in 1, depth 3, linear;
    fn test_minimax_find_mate_01() {
        let mut b = Board::parse_fen(&"5k2/5ppp/5PPP/8/8/8/4R3/4R1K1 w - - 0 1");
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 6, false, true);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "e2e8", true);
        assert_eq!(moves[0].eval.mate_in == 1, true);
    }

    #[test]
    // Mate in 1, depth 1, promotion;
    fn test_minimax_find_mate_02() {
        let mut b = Board::parse_fen(&"6k1/4Pppp/5P2/8/8/8/8/6K1 w - - 0 1");
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 2, true, true);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "e7e8q" || move_to_user(&b, &moves[0].mov) == "e7e8r", true);
        assert_eq!(moves[0].eval.mate_in == 1, true);
    }

    #[test]
    // Mate in 2, depth 2, promotion;
    fn test_minimax_find_mate_03() {
        let mut b = Board::parse_fen(&"6kq/5ppp/4P3/8/8/8/8/BB4K1 w - - 0 1");
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 4, true, true);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "e6e7", true);
        assert_eq!(moves[0].eval.mate_in == 2, true);
    }

    #[test]
    // Mate in 2, depth 2, forced Legal Mate sequence;
    fn test_minimax_find_mate_04() {
        let mut b = Board::parse_fen(&"r2qkbnr/ppp2ppp/2np4/4N3/2B1P3/2N4P/PPPP1PP1/R1BbK2R w KQkq - 0 7");
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 4, true, true);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "c4f7", true);
        assert_eq!(moves[0].eval.mate_in == 2, true);
    }

    #[test]
    // Mate in -1, depth 1, almost Fool's mate;
    fn test_minimax_find_mate_05() {
        let mut b = Board::parse_fen(&"rnbqkbnr/pppp1ppp/8/8/4pPP1/P7/1PPPP2P/RNBQKBNR b KQkq f3 0 3");
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 2, true, true);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "d8h4", true);
        assert_eq!(moves[0].eval.mate_in == -1, true);
    }

    #[test]
    // Mate in -3, depth 3, linear mate, but White is to move and lose;
    fn test_minimax_find_mate_06() {
        let mut b = Board::parse_fen(&"k3r3/3r4/8/8/8/8/8/5K2 w - - 0 1");
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 6, true, true);
        assert_eq!(moves[0].eval.mate_in == -3, true);
    }

    #[test]
    // Mate in 2, depth 3, Vukovic Mate #3 (lichess.org), position from Pavol Danek - Stanislav Hanuliak, 2001;
    fn test_minimax_find_mate_07() {
        let mut b = Board::parse_fen(&"2r5/8/8/5K1k/4N1R1/7P/8/8 w - - 12 67");
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 6, true, true);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "e4f6", true);
        assert_eq!(moves[0].eval.mate_in == 2, true);
    }

    #[test]
    // Mate in -2, depth 3, promotion with capture
    fn test_minimax_find_mate_08() {
        let mut b = Board::parse_fen(&"8/6N1/b7/8/6k1/3Q4/2pp1PPP/4B1K1 b - - 0 1");
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 6, true, true);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "d2e1q" || move_to_user(&b, &moves[0].mov) == "d2e1r", true);
        assert_eq!(moves[0].eval.mate_in == -2, true);
    }

    #[test]
    // Mate in 1, depth 2 long castle
    fn test_minimax_find_mate_09() {
        let mut b = Board::parse_fen(&"r2k1bnr/ppp1pppp/5N2/8/8/7B/PPP1PP1P/R3K1NR w KQ - 0 1");
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 4, true, true);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "e1c1", true);
        assert_eq!(moves[0].eval.mate_in == 1, true);
    }

    #[test]
    // Mate in -2, depth 2, short castle or Rf8
    fn test_minimax_find_mate_10() {
        let mut b = Board::parse_fen(&"rnb1k2r/pppp2pp/8/2b5/7Q/8/PPPPP1PP/1NBQRKR1 b kq - 0 1");
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 4, true, true);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "e8g8" || move_to_user(&b, &moves[0].mov) == "h8f8", true);
        assert_eq!(moves[0].eval.mate_in == -2, true);
    }

    #[test]
    // Mate in -1, depth 2, symmetrical, smoothered mate, board is in chaos
    fn test_minimax_find_mate_11() {
        let mut b = Board::parse_fen(&"qqq3rk/bbnP2pp/qqq2p2/4p1N1/4P1n1/QQQ2P2/BBNp2PP/QQQ3RK w - - 0 1");
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 4, true, true);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "g5f7", true);
        assert_eq!(moves[0].eval.mate_in == 1, true);
    }
    
    // Long tests that can be ignored, unless it is a perfomance test.

    #[test]
    #[ignore]
    // Mate in 3, depth 3, bishop and queen traps the castled king;
    fn test_minimax_find_long_mate_01() {
        let mut b = Board::parse_fen(&"4qrk1/p1r1Bppp/4b3/2p3Q1/8/3P4/PPP2PPP/R3R1K1 w - - 3 19");
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 6, true, true);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "e7f6", true);
        assert_eq!(moves[0].eval.mate_in == 3, true);
    }

    #[test]
    #[ignore]
    // Mate in -1, depth 3, almost Fool's mate;
    fn test_minimax_find_long_mate_02() {
        let mut b = Board::parse_fen(&"rnbqkbnr/pppp1ppp/8/8/4pPP1/P7/1PPPP2P/RNBQKBNR b KQkq f3 0 3");
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 6, true, true);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "d8h4", true);
        assert_eq!(moves[0].eval.mate_in == -1, true);
    }

    #[test]
    #[ignore]
    // Mate in 4, depth 4, first move is a castle;
    fn test_minimax_find_long_mate_03() {
        let mut b = Board::parse_fen(&"r1bk1bnr/ppp1pppp/5N2/8/8/7B/PPP1PP1P/R3K1NR w KQ - 0 1");
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 8, true, true);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "e1c1", true);
        assert_eq!(moves[0].eval.mate_in == 4, true);
    }
}