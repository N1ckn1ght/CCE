use std::collections::HashMap;

use crate::{engine::{character::Character, eval::{EvalMov, Eval, EvalHashed}, minimax::eval, hashtable::Hashtable}, board::board::Board};

pub struct Generic {
    // heatmap: [[[u64; 12]; 8]; 8]
    piece_costs     : [f32; 6],
    // k_threat        : f32,
    // k_allign        : f32,
    // k_mobility      : [f32; 5],

    // maximum static half-depth for minimax eval() call
    static_half_depth: i8,
    // number of a current eval() call for hash calculations from the last hash_clear()
    hash_iter: u8,
    // simple Zobrist hashtable
    hashtable: Hashtable,
    // stored evaluated board positions
    hashes: HashMap<u64, EvalHashed>
    // TODO: store already played positions on previous iterations without clearing them
    // use case - hash_clear() calls due to memory limitation
}

impl Character for Generic {
    fn get_eval_move(&mut self, board: &mut Board) -> EvalMov {
        self.get_eval_moves(board)[0]
    }

    fn get_eval_moves(&mut self, board: &mut Board) -> Vec<EvalMov> {
        let evals = eval(board, self, Eval::low(), Eval::high());
        self.hash_iter += 1;
        evals
    }

    fn set_static_half_depth(&mut self, half_depth: i8) {
        self.static_half_depth = half_depth;
    }

    fn get_static_eval(&self, board: &Board) -> f32 {
        let mut score: f32 = 0.0;
        for i in 0..8 {
            for j in 0..8 {
                if board.field[i][j] < 2 {
                    continue;
                }
                if board.field[i][j] & 1 == 1 {
                    score += self.piece_costs[((board.field[i][j] >> 1) - 1) as usize];
                } else {
                    score -= self.piece_costs[((board.field[i][j] >> 1) - 1) as usize];
                }
            }
        }
        score
    }

    fn get_static_eval_mate(&self, board: &Board) -> f32 {
        self.get_static_eval(board)
    }

    fn get_static_eval_stalemate(&self, board: &Board) -> f32 {
        0.0
    }

    fn get_static_half_depth(&self) -> i8 {
        self.static_half_depth
    }

    fn get_hash(&self, board: &Board) -> u64 {
        self.hashtable.hash(board)
    }

    fn hash_store(&mut self, hash: u64, eval: Eval, depth: i8) {
        if let Some(f) = self.hashes.get_mut(&hash) {
            // if is marked as played and thus already exists
            f.eval = eval;
            f.evaluated = true;
            f.depth = depth;
        } else {
            // if is not exists and thus not marked as played
            self.hashes.insert(hash, EvalHashed::evaluated(eval, depth, self.hash_iter, 0));
        }
    }

    fn hash_play(&mut self, hash: u64) {
        self.hashes.insert(hash, EvalHashed::new(self.hash_iter));
        // if let Some(f) = self.hashes.get_mut(&hash) {
        //     f.playcount += 1;
        // } else {
        //     self.hashes.insert(hash, EvalHashed::new(self.hash_iter));
        // }
    }

    fn hash_unplay(&mut self, hash: u64) {
        if let Some(f) = self.hashes.get_mut(&hash) {
            f.playcount -= 1;
        } else {
            // possibly bad hash_clear() call throught the minimax search iteration
            panic!("Attempt to 'unplay' non-existent position");
        }
    }

    fn is_played(&self, hash: u64) -> bool {
        if let Some(f) = self.hashes.get(&hash) {
            return f.playcount > 0;
        } else {
            return false;
        }
    }

    fn is_evaluated(&self, hash: u64) -> bool {
        if let Some(f) = self.hashes.get(&hash) {
            return f.evaluated;
        } else {
            return false;
        }
    }
    
    fn get_mutable_hashed_eval(&mut self, hash: u64) -> Option<&mut EvalHashed> {
        self.hashes.get_mut(&hash)
    }

    fn get_hash_iter(&self) -> u8 {
        self.hash_iter
    }

    fn clear_hashes(&mut self) {
        self.hashes.clear();
        self.hash_iter = 0;
    }
}

impl Generic {
    pub fn new() -> Self {
        // TODO: this should be passed as params
        let piece_costs = [1., 255., 3., 3., 4.5, 9.];
        Self { piece_costs, static_half_depth: 2, hash_iter: 0, hashtable: Hashtable::new(1024), hashes: HashMap::new() }
    }

    // fn count_moves_n() {
    //     // TODO
    // }

    // fn count_moves_bq() {
    //     // TODO
    // }

    // fn count_moves_rq() {
    //     // TODO
    // }

    // fn count_moves_k() {
    //     // TODO
    // }
}