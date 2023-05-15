use std::{collections::HashMap, cmp::{min, max}};

use crate::{engine::{character::Character, eval::{EvalMov, Eval, EvalHashed}, minimax::eval, hashtable::Hashtable}, board::{board::Board}};

pub struct Generic {
    // heatmap: [[[u64; 12]; 8]; 8]
    piece_costs     : [f32; 6],
    // k_threat        : f32,
    // k_allign        : f32,
    // k_mobility      : [f32; 5],

    // maximum static half-depth for minimax eval() call
    static_half_depth: i8,
    // maximum half-static half-depth for minimax eval() call
    mixed_half_depth: i8,
    // maximum dynamic half-depth for minimax eval() call
    dynamic_half_depth: i8,
    // simple Zobrist hashtable
    hashtable: Hashtable,
    // stored evaluated board positions (temporary cache)
    hashes_temp: HashMap<u64, EvalHashed>,
    // already played positions on board (permanent cache)
    hashes_perm: HashMap<u64, u8>,
    hashes_perm_history: Vec<u64>,
    // history of alpha/beta values used for minimax search (including next one to use)
    alpha_stack: Vec<Eval>,
    beta_stack:  Vec<Eval>,

    // temporary storage of evaluated moves
    // will clear itself on every accept_move() call
    evals: Vec<EvalMov>
}

impl Character for Generic {
    fn get_eval_move(&mut self, board: &mut Board) -> EvalMov {
        self.get_eval_moves(board)[0]
    }

    fn get_eval_moves(&mut self, board: &mut Board) -> &Vec<EvalMov> {
        self.evals = eval(board, self, *self.alpha_stack.last().unwrap(), *self.beta_stack.last().unwrap());
        // println!("\nDEBUG: alpha {}, beta {}\n", self.alpha_stack.last().unwrap().mate_in, self.beta_stack.last().unwrap().mate_in);
        &self.evals
    }

    fn set_static_half_depth(&mut self, half_depth: i8) {
        self.static_half_depth = half_depth;
    }

    fn set_mixed_half_depth(&mut self, half_depth: i8) {
        self.mixed_half_depth = half_depth;
    }

    fn set_dynamic_half_depth(&mut self, half_depth: i8) {
        self.dynamic_half_depth = half_depth;
    }

    fn accept_move(&mut self, board: &Board) {
        let mut found = false;
        let mov = board.history.last().unwrap().mov;
        // set next a/b values to push into search
        for eval_mov in &self.evals {
            if eval_mov.mov == mov {
                if eval_mov.eval.mate_in > 0 {
                    self.alpha_stack.push(Eval::low());
                    self.beta_stack.push(Eval::new(-Eval::BIG_SCORE, max(1, eval_mov.eval.mate_in - 1)));
                } else if eval_mov.eval.mate_in < 0 {
                    self.alpha_stack.push(Eval::new(Eval::BIG_SCORE, min(-1, eval_mov.eval.mate_in + 1)));
                    self.beta_stack.push(Eval::high());
                } else if !self.alpha_stack.is_empty() {
                    self.alpha_stack.push(Eval::low());
                    self.beta_stack.push(Eval::high());
                }
                found = true;
                break;
            }
        }
        if !found {
            if self.alpha_stack.last().unwrap().mate_in == 0 {
                self.alpha_stack.push(Eval::low());
                if self.beta_stack.last().unwrap().mate_in == 0 {
                    self.beta_stack.push(Eval::high());
                } else {
                    self.beta_stack.push(Eval::new(-Eval::BIG_SCORE, max(1, self.beta_stack.last().unwrap().mate_in - 1)));
                }
            } else {
                self.alpha_stack.push(Eval::new(Eval::BIG_SCORE, min(-1, self.alpha_stack.last().unwrap().mate_in + 1)));
            }
        }
        // deal with permanent cache
        let hash = self.hashtable.hash(board);
        self.hashes_perm_history.push(hash);
        if let Some(f) = self.hashes_perm.get_mut(&hash) {
            *f += 1;
        } else {
            self.hashes_perm.insert(hash, 1);
        }
        // deal with temporary cache
        self.evals.clear();
        self.hashes_temp.clear();
    }

    fn takeback(&mut self) {
        if self.hashes_perm_history.is_empty() {
            panic!("Attempt to make takeback from starting position");
        }
        let mut mark_for_delete = false;
        if let Some(f) = self.hashes_perm.get_mut(self.hashes_perm_history.last().unwrap()) {
            if *f == 1 {
                mark_for_delete = true;
            } else {
                *f -= 1;
            }
        }
        if mark_for_delete {
            self.hashes_perm.remove(self.hashes_perm_history.last().unwrap());
        }
        self.hashes_perm_history.pop();
        self.alpha_stack.pop();
        self.beta_stack.pop();
    }

    fn get_static_eval(&self, board: &Board) -> f32 {
        if board.hmw > 99 {
            return 0.0;
        }
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

    fn get_mixed_half_depth(&self) -> i8 {
        self.mixed_half_depth
    }

    fn get_dynamic_half_depth(&self) -> i8 {
        self.dynamic_half_depth
    }

    fn make_hash(&self, board: &Board) -> u64 {
        self.hashtable.hash(board)
    }

    fn cache_evaluated(&mut self, hash: u64, eval: Eval, depth: i8) {
        if let Some(f) = self.hashes_temp.get_mut(&hash) {
            // if is marked as played and thus already exists
            f.eval = eval;
            f.depth = depth;
        } else {
            // if is not exists and thus not marked as played
            self.hashes_temp.insert(hash, EvalHashed::evaluated(eval, depth, 0));
        }
    }

    fn cache_play(&mut self, hash: u64) {
        if let Some(f) = self.hashes_temp.get_mut(&hash) {
            f.playcount += 1;
        } else {
            self.hashes_temp.insert(hash, EvalHashed::new());
        }
    }

    fn cache_unplay(&mut self, hash: u64) {
        if let Some(f) = self.hashes_temp.get_mut(&hash) {
            f.playcount -= 1;
        } else {
            // possibly bad hash_clear() call throught the minimax search iteration
            panic!("Attempt to 'unplay' non-existent position");
        }
    }
    
    fn is_played(&self, hash: u64) -> bool {
        if let Some(f) = self.hashes_temp.get(&hash) {
            if f.playcount > 0 {
                return true;
            }
        }
        if let Some(f) = self.hashes_perm.get(&hash) {
            return *f > 1;
        }
        false
    }

    fn is_evaluated(&self, hash: u64) -> bool {
        if let Some(f) = self.hashes_temp.get(&hash) {
            return f.eval != Eval::unevaluated();
        } else {
            return false;
        }
    }

    fn get_hashed_eval(&self, hash: u64) -> Eval {
        self.hashes_temp.get(&hash).unwrap().eval
    }

    fn get_hashed_depth(&self, hash: u64) -> i8 {
        self.hashes_temp.get(&hash).unwrap().depth
    }
}

impl Generic {
    pub fn new(depths: &[i8; 3]) -> Self {
        // TODO: this should be passed as params
        let piece_costs = [1., 255., 3., 3., 4.5, 9.];
        Self { 
            piece_costs,
            static_half_depth: depths[0],
            mixed_half_depth: depths[1],
            dynamic_half_depth: depths[2],
            hashtable: Hashtable::new(1024),
            hashes_temp: HashMap::new(),
            hashes_perm: HashMap::new(),
            hashes_perm_history: Vec::default(),
            alpha_stack: [Eval::low()].to_vec(),
            beta_stack: [Eval::high()].to_vec(),
            evals: Vec::default()
        }
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

pub struct PermHashed {
    // move no
}