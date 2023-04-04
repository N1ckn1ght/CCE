use std::cmp::{max, min};
use crate::{board::{board::{Board, Check}, mov::{Mov}}};
use super::{eval::{EvalMov, Eval}, character::Character};

pub struct Minimax {
    called_half_depth: i8
}

impl Minimax {
    pub fn new() -> Self {
        Minimax { called_half_depth: 0 }
    }

    // this will copy the first minimax iteration
    // the purpose is to have a vector of evaluated possible moves as an output, not just the best one
    pub fn eval<Char: Character>(&mut self, board: &mut Board, char: &Char, half_depth: i8) -> Vec<EvalMov> {
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
                eval: self.minimax(board, char, half_depth - 1, Eval::low(), Eval::high(), board.white_to_move, board.get_check(&mov.data)) });
            board.revert_move();
        }

        if board.white_to_move {
            evals.sort_by(|a, b| b.eval.cmp(&a.eval));
        } else {
            evals.sort_by(|a, b| a.eval.cmp(&b.eval));
        }
        evals
    }

    // will return score eval and the mate_in moves if there's a forced checkmate sequence
    pub fn minimax<Char: Character>(&self, board: &mut Board, char: &Char, half_depth: i8, mut alpha: Eval, mut beta: Eval, maximize: bool, check: Check) -> Eval {
        // TODO: store positions!
        // TODO: check for hmw >= 50 or triple repetitive of a position

        // It might be even faster to check for half_depth first, but it just feels wrong
        // get_legal_moves is a relatively heavy method by now (which it shouldn't be however)
        // the fact that there are certain moves may be useful to get interesting static results

        let mut eval: Eval;
        let mut moves: Vec<Mov> = board.get_legal_moves(Some(check), Some(true));

        if moves.is_empty() {
            return match check {
                Check::InCheck | Check::InDoubleCheck => {
                    let mate_in = if maximize {
                        // White won
                        -(self.called_half_depth + 1 - half_depth) / 2
                    } else {
                        // Black won
                        (self.called_half_depth + 1 - half_depth) / 2
                    };
                    Eval {
                        score: char.static_eval(board, check, moves.len() as u8),
                        mate_in,
                    }
                }
                // Stalemate
                _ => Eval {
                    score: 0.0,
                    mate_in: 0,
                },
            };
        }

        // pre-sort in descending order by Mov data (will fasten a/b pruning)
        moves.sort_by(|a, b| b.data.cmp(&a.data));

        if half_depth < 1 {
            // TODO: run quiescense search first!
            return Eval { score: char.static_eval(board, check, moves.len() as u8), mate_in: 0 };
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

        eval
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
        let mut b = Board::parse_fen(&"5k2/5ppp/5PPP/8/8/8/4R3/4R1K1 w - - 0 1".to_string());
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 6);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "e2e8".to_string(), true);
        assert_eq!(moves[0].eval.mate_in == 1, true);
    }

    #[test]
    // Mate in 1, depth 1, promotion;
    fn test_minimax_find_mate_02() {
        let mut b = Board::parse_fen(&"6k1/4Pppp/5P2/8/8/8/8/6K1 w - - 0 1".to_string());
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 2);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "e7e8q".to_string() || move_to_user(&b, &moves[0].mov) == "e7e8r".to_string(), true);
        assert_eq!(moves[0].eval.mate_in == 1, true);
    }

    #[test]
    // Mate in 2, depth 2, promotion;
    fn test_minimax_find_mate_03() {
        let mut b = Board::parse_fen(&"6kq/5ppp/4P3/8/8/8/8/BB4K1 w - - 0 1".to_string());
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 4);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "e6e7".to_string(), true);
        assert_eq!(moves[0].eval.mate_in == 2, true);
    }

    #[test]
    // Mate in 2, depth 2, forced Legal Mate sequence;
    fn test_minimax_find_mate_04() {
        let mut b = Board::parse_fen(&"r2qkbnr/ppp2ppp/2np4/4N3/2B1P3/2N4P/PPPP1PP1/R1BbK2R w KQkq - 0 7".to_string());
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 4);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "c4f7".to_string(), true);
        assert_eq!(moves[0].eval.mate_in == 2, true);
    }

    #[test]
    // Mate in -1, depth 1, almost Fool's mate;
    fn test_minimax_find_mate_05() {
        let mut b = Board::parse_fen(&"rnbqkbnr/pppp1ppp/8/8/4pPP1/P7/1PPPP2P/RNBQKBNR b KQkq f3 0 3".to_string());
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 2);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "d8h4".to_string(), true);
        assert_eq!(moves[0].eval.mate_in == -1, true);
    }

    #[test]
    // Mate in -3, depth 3, linear mate, but White is to move and lose;
    fn test_minimax_find_mate_06() {
        let mut b = Board::parse_fen(&"k3r3/3r4/8/8/8/8/8/5K2 w - - 0 1".to_string());
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 6);
        assert_eq!(moves[0].eval.mate_in == -3, true);
    }

    #[test]
    // Mate in 2, depth 3, Vukovic Mate #3 (lichess.org), position from Pavol Danek - Stanislav Hanuliak, 2001;
    fn test_minimax_find_mate_07() {
        let mut b = Board::parse_fen(&"2r5/8/8/5K1k/4N1R1/7P/8/8 w - - 12 67".to_string());
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 6);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "e4f6".to_string(), true);
        assert_eq!(moves[0].eval.mate_in == 2, true);
    }

    #[test]
    // Mate in -2, depth 3, promotion with capture
    fn test_minimax_find_mate_08() {
        let mut b = Board::parse_fen(&"8/6N1/b7/8/6k1/3Q4/2pp1PPP/4B1K1 b - - 0 1".to_string());
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 6);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "d2e1q".to_string() || move_to_user(&b, &moves[0].mov) == "d2e1r".to_string(), true);
        assert_eq!(moves[0].eval.mate_in == -2, true);
    }

    #[test]
    // Mate in 1, depth 2 long castle
    fn test_minimax_find_mate_09() {
        let mut b = Board::parse_fen(&"r2k1bnr/ppp1pppp/5N2/8/8/7B/PPP1PP1P/R3K1NR w KQ - 0 1".to_string());
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 4);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "e1c1".to_string(), true);
        assert_eq!(moves[0].eval.mate_in == 1, true);
    }

    #[test]
    // Mate in -2, depth 2, short castle or Rf8
    fn test_minimax_find_mate_10() {
        let mut b = Board::parse_fen(&"rnb1k2r/pppp2pp/8/2b5/7Q/8/PPPPP1PP/1NBQRKR1 b kq - 0 1".to_string());
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 4);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "e8g8".to_string() || move_to_user(&b, &moves[0].mov) == "h8f8".to_string(), true);
        assert_eq!(moves[0].eval.mate_in == -2, true);
    }

    #[test]
    // Mate in -1, depth 2, double check mate, board is in chaos
    fn test_minimax_find_mate_11() {
        let mut b = Board::parse_fen(&"qqq3rk/bbnP2pp/qqq2p2/4p1N1/4P1n1/QQQ2P2/BBNp2PP/QQQ3RK w - - 0 1".to_string());
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 4);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "g5f7".to_string(), true);
        assert_eq!(moves[0].eval.mate_in == 1, true);
    }
}