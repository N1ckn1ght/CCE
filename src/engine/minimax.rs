use std::{cmp::{max, min, Ordering}, collections::HashMap};
use crate::{board::{board::{Board, Check}, mov::{Mov}}};
use super::{eval::{EvalMov, Eval}, character::Character, hashtable::Hashtable};

pub struct Minimax {
    called_half_depth: i8,
    no: u8,
    hashtable: Hashtable,
    hashes: HashMap<u64, EvalHashed>
}

impl Minimax {
    pub fn new() -> Self {
        Self { called_half_depth: 0, no: 0, hashtable: Hashtable::new(2005), hashes: HashMap::new() }
    }

    // this will copy the first minimax iteration
    // the purpose is to have a vector of evaluated possible moves as an output, not just the best one
    // initial alpha and beta are recommended to be just Eval::high()
    pub fn eval<Char: Character>(&mut self, board: &mut Board, char: &Char, half_depth: i8, mut alpha: Eval, mut beta: Eval, drop_hash: bool) -> Vec<EvalMov> {
        if drop_hash {
            self.hashes.clear();
            self.no = 0;
        }
        if half_depth < 1 {
            panic!("0-half-depth evaluation attempt");
        }
        
        self.called_half_depth = half_depth;
        
        let mut moves: Vec<Mov> = board.get_legal_moves(Some(Check::Unknown), Some(true));
        let mut evals: Vec<EvalMov> = Vec::default();
        
        // pre-sort in descending order by Mov data (will fasten a/b pruning)
        moves.sort_by(|a, b| b.data.cmp(&a.data));

        for mov in &moves {
            board.make_move(mov);
            evals.push(EvalMov{ 
                mov: *mov, 
                eval: self.minimax(board, char, half_depth - 1, alpha, beta, board.white_to_move, board.get_check(&mov.data)) });
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

        self.no += 1;

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
        if let Some(evaluated) = self.hashes.get_mut(&hash) {
            let delta = self.called_half_depth - half_depth - evaluated.depth;
            match delta.cmp(&0){
                // This means we found a quick way to get already evaluated position!
                Ordering::Less => {
                    match evaluated.eval.mate_in.cmp(&0) {
                        Ordering::Less => {
                            if self.no == evaluated.no {
                                evaluated.eval.mate_in -= delta;
                            }
                            return evaluated.eval;
                        },
                        Ordering::Equal => {
                            // Just calculate in more depth, there could be a mate in something
                            // or just a better or a worse position than expected
                        },
                        Ordering::Greater => {
                            if self.no == evaluated.no {
                                evaluated.eval.mate_in += delta;
                            }
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
                            if self.no == evaluated.no {
                                return Eval { score: evaluated.eval.score, mate_in: evaluated.eval.mate_in - delta };
                            } else {
                                return Eval { score: evaluated.eval.score, mate_in: evaluated.eval.mate_in };
                            }
                        },
                        Ordering::Equal => {
                            // TODO: analyze if it's a draw or a repetitive move
                            return evaluated.eval;
                        }
                        Ordering::Greater => {
                            if self.no == evaluated.no {
                                return Eval { score: evaluated.eval.score, mate_in: evaluated.eval.mate_in + delta };
                            } else {
                                return Eval { score: evaluated.eval.score, mate_in: evaluated.eval.mate_in };
                            }
                        }
                    }
                }
            }
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
            self.hashes.insert(hash, EvalHashed::new(eval, self.called_half_depth - half_depth, self.no));
            return eval;
        }

        // pre-sort in descending order by Mov data (will fasten a/b pruning)
        moves.sort_by(|a, b| b.data.cmp(&a.data));

        if half_depth < 1 {
            // TODO: run quiescense search first!
            let eval = Eval { score: char.static_eval(board), mate_in: 0 };
            self.hashes.insert(hash, EvalHashed::new(eval, self.called_half_depth, self.no));
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

        self.hashes.insert(hash, EvalHashed::new(eval, self.called_half_depth - half_depth, self.no));
        eval
    }
}

// Sum: 64 + 8 + 8 + 8 = 88 bit (per hashed position)
pub struct EvalHashed {
    pub eval: Eval,
    pub depth: i8,
    pub no: u8,
    pub played: u8
}

impl EvalHashed {
    pub fn new(eval: Eval, depth: i8, no: u8) -> Self {
        Self { eval, depth, no, played: 0 }
    }

    pub fn play(&mut self) {
        self.played += 1;
    }

    pub fn unplay(&mut self) {
        self.played -= 1;
    }

    pub fn played(& self, threshold: u8) -> bool {
        self.played >= threshold
    }
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
        let moves = engine.eval(&mut b, &Materialist{}, 6, Eval::lower(), Eval::higher(), true);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "e2e8", true);
        assert_eq!(moves[0].eval.mate_in == 1, true);
    }

    #[test]
    // Mate in 1, depth 1, promotion;
    fn test_minimax_find_mate_02() {
        let mut b = Board::parse_fen(&"6k1/4Pppp/5P2/8/8/8/8/6K1 w - - 0 1");
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 2, Eval::low(), Eval::high(), true);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "e7e8q" || move_to_user(&b, &moves[0].mov) == "e7e8r", true);
        assert_eq!(moves[0].eval.mate_in == 1, true);
    }

    #[test]
    // Mate in 2, depth 2, promotion;
    fn test_minimax_find_mate_03() {
        let mut b = Board::parse_fen(&"6kq/5ppp/4P3/8/8/8/8/BB4K1 w - - 0 1");
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 4, Eval::low(), Eval::high(), true);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "e6e7", true);
        assert_eq!(moves[0].eval.mate_in == 2, true);
    }

    #[test]
    // Mate in 2, depth 2, forced Legal Mate sequence;
    fn test_minimax_find_mate_04() {
        let mut b = Board::parse_fen(&"r2qkbnr/ppp2ppp/2np4/4N3/2B1P3/2N4P/PPPP1PP1/R1BbK2R w KQkq - 0 7");
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 4, Eval::low(), Eval::high(), true);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "c4f7", true);
        assert_eq!(moves[0].eval.mate_in == 2, true);
    }

    #[test]
    // Mate in -1, depth 1, almost Fool's mate;
    fn test_minimax_find_mate_05() {
        let mut b = Board::parse_fen(&"rnbqkbnr/pppp1ppp/8/8/4pPP1/P7/1PPPP2P/RNBQKBNR b KQkq f3 0 3");
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 2, Eval::low(), Eval::high(), true);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "d8h4", true);
        assert_eq!(moves[0].eval.mate_in == -1, true);
    }

    #[test]
    // Mate in -3, depth 3, linear mate, but White is to move and lose;
    fn test_minimax_find_mate_06() {
        let mut b = Board::parse_fen(&"k3r3/3r4/8/8/8/8/8/5K2 w - - 0 1");
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 6, Eval::new(0.0, -3), Eval::high(), true);
        assert_eq!(moves[0].eval.mate_in == -3, true);
    }

    #[test]
    // Mate in 2, depth 3, Vukovic Mate #3 (lichess.org), position from Pavol Danek - Stanislav Hanuliak, 2001;
    fn test_minimax_find_mate_07() {
        let mut b = Board::parse_fen(&"2r5/8/8/5K1k/4N1R1/7P/8/8 w - - 12 67");
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 6, Eval::low(), Eval::high(), true);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "e4f6", true);
        assert_eq!(moves[0].eval.mate_in == 2, true);
    }

    #[test]
    // Mate in -2, depth 3, promotion with capture
    fn test_minimax_find_mate_08() {
        let mut b = Board::parse_fen(&"8/6N1/b7/8/6k1/3Q4/2pp1PPP/4B1K1 b - - 0 1");
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 6, Eval::low(), Eval::high(), true);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "d2e1q" || move_to_user(&b, &moves[0].mov) == "d2e1r", true);
        assert_eq!(moves[0].eval.mate_in == -2, true);
    }

    #[test]
    // Mate in 1, depth 2 long castle
    fn test_minimax_find_mate_09() {
        let mut b = Board::parse_fen(&"r2k1bnr/ppp1pppp/5N2/8/8/7B/PPP1PP1P/R3K1NR w KQ - 0 1");
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 4, Eval::low(), Eval::high(), true);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "e1c1", true);
        assert_eq!(moves[0].eval.mate_in == 1, true);
    }

    #[test]
    // Mate in -2, depth 2, short castle or Rf8
    fn test_minimax_find_mate_10() {
        let mut b = Board::parse_fen(&"rnb1k2r/pppp2pp/8/2b5/7Q/8/PPPPP1PP/1NBQRKR1 b kq - 0 1");
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 4, Eval::low(), Eval::high(), true);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "e8g8" || move_to_user(&b, &moves[0].mov) == "h8f8", true);
        assert_eq!(moves[0].eval.mate_in == -2, true);
    }

    #[test]
    // Mate in -1, depth 2, symmetrical, smoothered mate, board is in chaos
    fn test_minimax_find_mate_11() {
        let mut b = Board::parse_fen(&"qqq3rk/bbnP2pp/qqq2p2/4p1N1/4P1n1/QQQ2P2/BBNp2PP/QQQ3RK w - - 0 1");
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 4, Eval::low(), Eval::high(), true);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "g5f7", true);
        assert_eq!(moves[0].eval.mate_in == 1, true);
    }
    
    #[test]
    // Mate in -1, depth 3, almost Fool's mate;
    fn test_minimax_find_long_mate_01() {
        let mut b = Board::parse_fen(&"rnbqkbnr/pppp1ppp/8/8/4pPP1/P7/1PPPP2P/RNBQKBNR b KQkq f3 0 3");
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 6, Eval::low(), Eval::high(), true);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "d8h4", true);
        assert_eq!(moves[0].eval.mate_in == -1, true);
    }

    #[test]
    // Mate in 4, depth 4, first move is a castle;
    fn test_minimax_find_long_mate_02() {
        let mut b = Board::parse_fen(&"r1bk1bnr/ppp1pppp/5N2/8/8/7B/PPP1PP1P/R3K1NR w KQ - 0 1");
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 8, Eval::low(), Eval::high(), true);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "e1c1", true);
        assert_eq!(moves[0].eval.mate_in == 4, true);
    }

    #[test]
    // Mate in 5, depth 4, use hashing, mate is kinda forced;
    fn test_minimax_find_hash_mate_01() {
        let mut b = Board::parse_fen(&"2q1nk1r/4Rp2/1ppp1P2/6Pp/3p1B2/3P3P/PPP1Q3/6K1 w - - 0 1");
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 8, Eval::low(), Eval::high(), true);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "e7e8", true);
        assert_eq!(moves[0].eval.mate_in == 5, true);
    }

    // Long tests that can be ignored, unless it is a perfomance test.

    #[test]
    #[ignore]
    // Mate in 3, depth 3, bishop and queen traps the castled king;
    fn test_minimax_find_quiet_mate_01() {
        let mut b = Board::parse_fen(&"4qrk1/p1r1Bppp/4b3/2p3Q1/8/3P4/PPP2PPP/R3R1K1 w - - 3 19");
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 6, Eval::low(), Eval::high(), true);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "e7f6", true);
        assert_eq!(moves[0].eval.mate_in == 3, true);
    }

    #[test]
    #[ignore]
    // Mate in 5, depth 4, use hashing, mate is kinda forced;
    fn test_minimax_find_long_mate_03() {
        let mut b = Board::parse_fen(&"6k1/3b3r/1p1p4/p1n2p2/1PPNpP1q/P3Q1p1/1R1RB1P1/5K2 b - - 0 1");
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 10, Eval::low(), Eval::high(), true);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "h4f4", true);
        assert_eq!(moves[0].eval.mate_in == -5, true);
    }
}