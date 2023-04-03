use crate::{engine::character::Character, board::board::{Board, Check}};

pub struct Materialist;

impl Character for Materialist {
    fn static_eval(&self, board: &Board, check: Check, legal_moves_count: u8) -> f32 {
        let mut score: f32 = 0.0;
        for i in 0..8 {
            for j in 0..8 {
                if board.field[i][j] < 2 {
                    continue;
                }
                // TODO: match is in fact slow, it might be good to have a hashmap here?
                match board.gpr(&board.field[i][j]) {
                    'p' => score -= 1.0,
                    'P' => score += 1.0,
                    'n' => score -= 3.0,
                    'N' => score += 3.0,
                    'b' => score -= 3.0,
                    'B' => score += 3.0,
                    'r' => score -= 4.5,
                    'R' => score += 4.5,
                    'q' => score -= 9.0,
                    'Q' => score += 9.0,
                    'k' => score -= 255.0,
                    'K' => score += 255.0,
                    _ => panic!("Unknown piece found, value = {}", board.field[i][j])
                }
            }
        }

        score
    }
}