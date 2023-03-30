use crate::board::board::Board;

const BIG_NUMBER: f32 = 1048576.0;

pub struct Minimax {
    // store general idea?
    // store hash of positions
    // etc
}

impl Minimax {
    pub fn eval(board: &Board, depth: u8) {
        let moves: Vec<Mov> = board.get_legal_moves();
        let evals: Vec<EvalMov> = vec![];
        let maximize: bool = board.white_to_move;
        
        // sort in descending order by Mov data
        moves.sort_by(|a, b| b.data.cmp(&a.data));
        for mov in &moves {
            board.make_move(&mov);
            // evals.push(EvalMov{mov, eval:})
        }
    }

    // will return eval and the mate_in moves if there's a forced checkmate sequence
    pub fn minimax(board: &Board, depth: u8, alpha: f32, beta: f32, maximize: bool, check: Check) -> (f32, i8) {
        // or no moves are possible?
        if depth < 1 {
            // return static eval
            return 0.0
        }

        let mut eval: f32 = 0;
        let moves: Vec<Mov> = board.get_legal_moves();
        if maximize {
            eval = -BIG_NUMBER;

        } else {
            eval = BIG_NUMBER;
        }

        0.0
    }
}