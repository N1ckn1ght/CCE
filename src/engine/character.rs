use crate::board::{board::Board, mov::Mov};
use super::eval::EvalMov;

pub trait Character {
    // TODO: determine any additional necessary parameters to squeeze at least some speed
    fn static_eval(&self, board: &Board) -> f32;

    // Evaluate it to choose less painful mate later, or just return any value
    fn static_eval_mate(&self, board: &Board) -> f32;

    // Return the best move out, or maybe the situation/difficulty-suited move.
    // Evals are expected to be pre-sorted in favour of a current color to move
    fn get_move(&mut self, board: &Board, evals: &[EvalMov]) -> Mov;
}