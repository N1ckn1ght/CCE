use crate::board::board::Board;

pub trait Character {
    fn static_eval(&self, board: &Board) -> f32;
}