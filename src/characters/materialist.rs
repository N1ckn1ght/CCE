use crate::{engine::{character::Character, eval::EvalMov}, board::{board::Board, mov::Mov}};

pub struct Materialist;

impl Character for Materialist {
    fn static_eval(&self, board: &Board) -> f32 {
        let mut score: f32 = 0.0;
        for i in 0..8 {
            for j in 0..8 {
                if board.field[i][j] < 2 {
                    continue;
                }
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

    fn static_eval_mate(&self, board: &Board) -> f32 {
        self.static_eval(board)
    }

    fn get_move(&mut self, _board: &Board, evals: &[EvalMov]) -> Mov {
        evals[0].mov
    }
}