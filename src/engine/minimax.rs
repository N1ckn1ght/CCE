use std::cmp::{max, min};
use crate::board::{board::{Board, Check}, mov::{Mov}};
use super::{eval::{EvalMov, Eval}, character::Character};

pub struct Minimax {
    // store general idea?
    // store hash of positions
    // etc
}

impl Minimax {
    // this will copy the first minimax iteration
    // the purpose is to have a vector of evaluated possible moves as an output, not just the best one
    pub fn eval<Char: Character>(board: &mut Board, char: &Char, half_depth: u8) -> Vec<EvalMov> {
        if half_depth == 0 {
            panic!("0-half-depth evaluation attempt");
        }
        
        let mut moves: Vec<Mov> = board.get_legal_moves(Some(Check::Unknown), Some(true));
        let mut evals: Vec<EvalMov> = vec![];
        
        // sort in descending order by Mov data
        moves.sort_by(|a, b| b.data.cmp(&a.data));
        for mov in &moves {
            board.make_move(*mov);
            evals.push(EvalMov{ 
                mov: *mov, 
                eval: Self::minimax(board, char, half_depth - 1, Eval::lowest(), Eval::highest(), board.white_to_move, board.bit_to_check(&mov.data)) });
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
    pub fn minimax<Char: Character>(board: &mut Board, char: &Char, half_depth: u8, mut alpha: Eval, mut beta: Eval, maximize: bool, check: Check) -> Eval {
        // TODO: store positions!
        // TODO: check for hmw >= 50 or triple repetitive of a position

        // It might be even faster to check for half_depth first, but it just feels wrong
        // get_legal_moves is a relatively heavy method by now (which it shouldn't be however)
        // the fact that there are certain moves may be useful to get interesting static results

        let mut eval: Eval;
        let moves: Vec<Mov> = board.get_legal_moves(Some(check), Some(true));
        
        // No moves? check for victory / stalemate

        if moves.len() == 0 {
            if check == Check::InCheck || check == Check::InDoubleCheck {
                if maximize {
                    return Eval::lowest();
                } else {
                    return Eval::highest();
                }
            } else {
                // pure draw case: { 0.0, M0 }
                return Eval::empty();
            }
        }

        if half_depth < 1 {
            // TODO: run quiescense search first!
            return Eval { score: char.static_eval(board), mate_in: 0 };
        }
        
        if maximize {
            eval = Eval::lowest();
            for mov in &moves {
                board.make_move(*mov);
                let temp = Self::minimax(board, char, half_depth - 1, alpha, beta, board.white_to_move, board.bit_to_check(&mov.data));
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
                let temp = Self::minimax(board, char, half_depth - 1, alpha, beta, board.white_to_move, board.bit_to_check(&mov.data));
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