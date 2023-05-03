use crate::board::board::Board;
use super::eval::{EvalMov, Eval};

pub trait Character {
    //
    // Calls from engine:

    // Return the move to make with its eval
    fn get_eval_move(&mut self, board: &Board) -> EvalMov;

    //
    // Calls from minimax:

    // Return the static evaluation of the position
    fn get_static_eval(&self, board: &Board) -> f32;
    // Return the static evaluation of a guaranteed win/lose position
    fn get_static_eval_mate(&self, board: &Board) -> f32;

    // Position hashing features

    // Store evaluated hash position
    fn hash_store(hash: u64, eval: Eval, depth: i8);
    // Store position as played
    fn hash_play(hash: u64, depth: i8);
    // Remove position from being played
    fn hash_unplay(hash: u64);
    // Check if this position was reached before
    fn is_played(hash: u64);
    // Clear all stored hashes
    fn hash_clear();
}

// Sum: 64 + 8 + 8 + 8 = 88 bit (per hashed position)
pub struct EvalHashed {
    pub eval: Eval,
    pub depth: i8,
    pub no: u8,
    pub played: bool,
    pub evaluated: bool
}

impl EvalHashed {
    pub fn new() -> Self {
        Self { eval: Eval::empty(), depth: 0, no: 0, played: true, evaluated: false } 
    }

    pub fn evaluated(eval: Eval, depth: i8, no: u8, played: bool) -> Self {
        Self { eval, depth, no, played, evaluated: true }
    }
}