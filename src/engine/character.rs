use crate::board::board::Board;
use super::eval::{EvalMov, Eval, EvalHashed};

pub trait Character {
    //
    // Calls from engine:

    // Return the move to make with its evaluation
    fn get_eval_move(&mut self, board: &mut Board) -> EvalMov;
    //
    fn get_eval_moves(&mut self, board: &mut Board) -> Vec<EvalMov>;
    //
    fn set_static_half_depth(&mut self, half_depth: i8);

    //
    // Calls from minimax:

    // Return the static evaluation of the position
    fn get_static_eval(&self, board: &Board) -> f32;
    // Return the static evaluation of a guaranteed win/lose position
    fn get_static_eval_mate(&self, board: &Board) -> f32;
    // Return the static evaluation of a guaranteed draw position
    fn get_static_eval_stalemate(&self, board: &Board) -> f32;
    // Return the maximum static half-depth param called on this minimax evaluation
    fn get_static_half_depth(&self) -> i8;

    // Position hashing features

    // Hash given position on board
    fn get_hash(&self, board: &Board) -> u64;
    // Store evaluated position
    fn hash_store(&mut self, hash: u64, eval: Eval, depth: i8);
    // Store non-evaluated position as played
    // ! This will erase the eval and playcount if it's already stored
    fn hash_play(&mut self, hash: u64);
    // Remove 1 playcount from position
    fn hash_unplay(&mut self, hash: u64);
    fn is_played(&self, hash: u64) -> bool;
    fn is_evaluated(&self, hash: u64) -> bool;
    // Get mutable hash to work with
    // ! While it's may be considered dangerous to work directly and not through the methods above,
    //   this should increase an overall perfomance, so worth it
    fn get_mutable_hashed_eval(&mut self, hash: u64) -> Option<&mut EvalHashed>;
    // Get current hashing iteration
    fn get_hash_iter(&self) -> u8;
    // Clear all stored hashes
    fn clear_hashes(&mut self);
}