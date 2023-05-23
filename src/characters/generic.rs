use std::{collections::HashMap, cmp::{min, max}};

use crate::{engine::{character::Character, eval::{EvalMov, Eval, EvalHashed}, minimax::eval, hashtable::Hashtable}, board::{board::Board}};

pub struct Generic {
    weights: GenericWeights,

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
        let depths = [self.static_half_depth, self.mixed_half_depth, self.dynamic_half_depth];

        if self.hashes_perm_history.is_empty() {
            let hash = self.make_hash(board);
            self.hashes_perm_history.push(hash);
            self.hashes_perm.insert(hash, 1);
        }

        // TODO: write a good code?
        // This is wrong because it mess with the depth and doesn't tell user about it.
        // However this will yield instant results instead of pondering already evaluated positions, also mate enclosure.
        // May provoke something unprecedented.
        let mut mate_in = 0;
        if self.alpha_stack.last().unwrap().mate_in != -Eval::BIG_MATE {
            mate_in = -self.alpha_stack.last().unwrap().mate_in;
            // println!("debug, alpha mate_in (abs): {}", mate_in);
        } else if self.beta_stack.last().unwrap().mate_in != Eval::BIG_MATE {
            mate_in = self.beta_stack.last().unwrap().mate_in;
            // println!("debug,  beta mate_in (   ): {}", mate_in);
        }
        if mate_in != 0 {
            self.static_half_depth = mate_in;
            self.mixed_half_depth = mate_in;
            self.dynamic_half_depth = mate_in;
        }

        self.evals = eval(board, self, *self.alpha_stack.last().unwrap(), *self.beta_stack.last().unwrap());
        self.static_half_depth = depths[0];
        self.mixed_half_depth = depths[1];
        self.dynamic_half_depth = depths[2];
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
            let a = self.alpha_stack.last().unwrap().mate_in;
            let b = self.beta_stack.last().unwrap().mate_in;
            if a == 0 || a == -Eval::BIG_MATE {
                self.alpha_stack.push(Eval::low());
                if b == 0 || b == Eval::BIG_MATE {
                    self.beta_stack.push(Eval::high());
                } else {
                    found = true;
                    self.beta_stack.push(Eval::new(-Eval::BIG_SCORE, max(1, self.beta_stack.last().unwrap().mate_in - 1)));
                }
            } else {
                found = true;
                self.alpha_stack.push(Eval::new(Eval::BIG_SCORE, min(-1, self.alpha_stack.last().unwrap().mate_in + 1)));
                self.beta_stack.push(Eval::high());
            }
        }
        // deal with permanent cache
        let hash = self.make_hash(board);
        self.hashes_perm_history.push(hash);
        if let Some(f) = self.hashes_perm.get_mut(&hash) {
            *f += 1;
        } else {
            self.hashes_perm.insert(hash, 1);
        }
        // deal with temporary cache
        // TODO: mate_in will go crazy, find a way around to implement hashing in between moves
        // if !found {
        if mov.is_repeatable(board) {
            self.hashes_temp.remove(&hash);
        } else {
            self.hashes_temp.clear();
        }
        // }
        self.evals.clear(); 
    }

    fn takeback(&mut self) {
        // < 2 is because starting position is also stored in history
        if self.hashes_perm_history.len() < 2 {
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
        self.hashes_temp.clear();
    }

    fn get_static_eval(&self, board: &Board) -> f32 {
        if board.hmw > 99 {
            return 0.0;
        }
        
        let mut score: f32 = 0.0;
        // 
        let mut white_pawn_islands: f32 = 1.0;
        let mut black_pawn_islands: f32 = 1.0;
        let scanned = [[false; 8]; 8];

        for i in 0..8 {
            for j in 0..8 {
                if scanned[i][j] || board.field[i][j] < 2 {
                    continue;
                }
                let mut piece = board.field[i][j];
                let color_bit = piece & 1;
                piece &= 254;

                // TODO: match by symbols (board.gpl() usage preferred)
                match piece {
                    // pawn
                    2 => {
                        
                    },
                    // king
                    4 => (),
                    // knight
                    6 => (),
                    // bishop
                    8 => (),
                    // rook
                    10 => {
                        score += self.scan_for_rq(board, &scanned, i as u8, j as u8, color_bit, 1.0);
                    }
                    // queen
                    12 => (),
                    //
                    _ => ()
                }

                //
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
    pub fn new(weights: GenericWeights, depths: &[i8; 3]) -> Self {
        // TODO: this should be passed as params
        let piece_costs = [1., 255., 3., 3., 4.5, 9.];
        Self { 
            weights,
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

    fn scan_fror_n() {
        todo!();
    }

    fn scan_for_bq() {
        todo!();
    }

    fn scan_for_rq(&self, board: &Board, scanned: &[[bool; 8]; 8], y: u8, x: u8, color_bit: u8, mult: f32) -> f32 {
        scanned[y as usize][x as usize] = true;
        
        // let mut static_value = 0.0;
        // let mut dynamic_value = 0.0;
        // let mut piece = board.field[y as usize][x as usize];
        // let mut temp = 0.0;
        // static_value += self.weights.material_cost[&board.gpr(&(piece - color_bit))];
        // static_value + dynamic_value

        0.0
    }

    fn scan_for_rq_vertical(&self, board: &Board, scanned: &[[bool; 8]; 8], y: u8, x: u8, color_bit: u8, mult: f32) -> f32 {
        let mut piece = board.field[y as usize][x as usize];
        let mut i = 1;
        
        let mut static_value = self.weights.material_cost[&board.gpr(&(piece - color_bit))];
        let mut dynamic_value = 0.0;

        while Board::in_bound_single(y + i, 0) {
            piece = board.field[(y + i) as usize][x as usize];
            i += 1;
            if piece < 2 {
                continue;
            }
            if piece % 254 == color_bit && !scanned[(y + i) as usize][x as usize] {
                if piece == board.gpl(&'r') + color_bit {
                    static_value += self.scan_for_rq_horizontal(board, scanned, y + i, x, color_bit, mult);
                } else if piece == board.gpl(&'q') + color_bit {
                    static_value += self.scan_for_rq_horizontal(board, scanned, y + i, x, color_bit, mult);
                    static_value += self.scan_for_bq(board, scanned, y + i, x, color_bit, mult);    
                } else {

                }
            } else {

            }
        }
        i = 1;
        while Board::in_bound_single(y, i) {
            piece = board.field[(y + i) as usize][x as usize];
            i += 1;
            if piece < 2 {
                continue;
            }

        }

        0.0
    }

    fn scan_for_rq_horizontal(&self, board: &Board, scanned: &[[bool; 8]; 8], y: u8, x: u8, color_bit: u8, mult: f32) -> f32 {
        
        0.0
    }

    fn scan_for_k() {
        todo!();
    }
}

pub struct GenericWeights {
    // pknbrq
    material_cost: HashMap<char, f32>,
    // knbrq
    mobility_k: HashMap<char, f32>,
    // amount of cells
    mobility_min_threshold: HashMap<char, f32>,
    mobility_max_threshold: HashMap<char, f32>,
    //
    pawn_islands_penalty: f32,
    pawn_advanced_multiplier: f32,
    pawn_passed_multiplier: f32,
    battery_vertical_cost: f32,
    battery_horizontal_cost: f32,
    battery_diagonal_cost: f32,
    battery_xray_self_multiplier: f32,
    battery_xray_opponent_multiplier: f32,
    battery_king_threat_multiplier: f32,
    defend_cheaper_piece_multiplier: f32,
    // king_safety_k: f32,
    k_kvb: f32    
}

impl GenericWeights {
    pub fn new() -> Self {
        todo!();
    }
}