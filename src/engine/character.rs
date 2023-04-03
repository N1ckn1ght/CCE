use crate::board::board::{Board, Check};

pub trait Character {
    // Score-evaluate a hopeless position, or not?
    // This burden of a decision will lie upon a character, not the engine itself.
    fn static_eval(&self, board: &Board, check: Check, legal_moves_count: u8) -> f32;
}