use crate::board::{board::Board, mov::Mov};
use super::eval::{EvalMov, Eval, EvalHashed};

pub trait Character {
    //
    // Calls from engine:

    // Return the move to make with its evaluation
    fn get_eval_move(&mut self, board: &mut Board) -> EvalMov;
    // Return the list of all evaluated moves
    fn get_eval_moves(&mut self, board: &mut Board) -> &Vec<EvalMov>;
    // Sets maximum static half-depth for minimax search
    fn set_static_half_depth(&mut self, half_depth: i8);
    // Set move as played in game; send board with the move that is already made
    fn accept_move(&mut self, board: &Board);
    // 
    fn takeback(&mut self);

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
    fn make_hash(&self, board: &Board) -> u64;
    // Delete temprorary hashes
    fn clear_cache(&mut self);
    // Store evaluated position
    fn cache_evaluated(&mut self, hash: u64, eval: Eval, depth: i8, iter_check: bool);
    // Add 1 playcount to the position; add as unevaluated, if missing
    fn cache_play(&mut self, hash: u64);
    // Remove 1 playcount from the position; if non-existant, will panic!
    fn cache_unplay(&mut self, hash: u64);
    // Return true if the position was played enough times - 1 to call it a draw (1 or 2)
    fn is_played(&self, hash: u64) -> bool;
    // Return true if the position is evaluated
    fn is_evaluated(&self, hash: u64) -> bool;
    // Get evaluation from given hashed position (will panic if no such position)
    fn get_hashed_eval(&self, hash: u64) -> Eval;
    // Get depth value on which given position was evaluated (will panic if no such position)
    fn get_hashed_depth(&self, hash: u64) -> i8;
}