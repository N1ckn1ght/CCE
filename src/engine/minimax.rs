use std::cmp::{max, min};
use crate::{board::{board::{Board, Check}, mov::{Mov}}, utils::utils::move_to_user};
use super::{eval::{EvalMov, Eval}, character::Character};

pub struct Minimax {
    called_half_depth: i8
}

impl Minimax {
    pub fn new() -> Minimax {
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
        let mut evals: Vec<EvalMov> = vec![];
        
        // pre-sort in descending order by Mov data (will fasten a/b pruning)
        moves.sort_by(|a, b| b.data.cmp(&a.data));

        for mov in &moves {
            board.make_move(*mov);
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

        // No moves? check for victory / stalemate
        if moves.len() == 0 {
            if check == Check::InCheck || check == Check::InDoubleCheck {
                if maximize {
                    // White won
                    return Eval { score: char.static_eval(board, check, moves.len() as u8), mate_in: - (self.called_half_depth + 1 - half_depth) / 2 }
                } else {
                    // Black won
                    return Eval { score: char.static_eval(board, check, moves.len() as u8), mate_in:   (self.called_half_depth + 1 - half_depth) / 2 }
                }
            } else {
                // Stalemate
                return Eval { score: 0.0, mate_in: 0 };
            }
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
                board.make_move(*mov);
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
                board.make_move(*mov);
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