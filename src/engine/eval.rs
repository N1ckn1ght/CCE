use std::cmp::Ordering;
use crate::board::mov::Mov;

// if there's a winning sequence, we should look onto the fastest possible, not on the score
#[derive(Clone, Copy)]
pub struct Eval {
    pub score: f32,
    pub mate_in: i8
}

impl Eval {
    const BIG_SCORE: f32 = 1048576.0;
    const BIG_MATE_IN: i8 = 127;

    pub fn new(score: f32, mate_in: i8) -> Eval {
        Eval { score, mate_in }
    }

    pub fn empty() -> Eval {
        Eval { score: 0.0, mate_in: 0 }
    }

    pub fn highest() -> Eval {
        Eval { score: Eval::BIG_SCORE, mate_in: Eval::BIG_MATE_IN }
    }

    pub fn lowest() -> Eval {
        Eval { score: -Eval::BIG_SCORE, mate_in: -Eval::BIG_MATE_IN }
    }
}

impl PartialEq for Eval {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score && self.mate_in == other.mate_in
    }
}

impl Eq for Eval {}

impl PartialOrd for Eval {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // expected: -3 -2 -1 ... 1 2 3
        if self.mate_in == other.mate_in {
            if self.score == other.score {
                return Some(Ordering::Equal);
            }
            if self.score < other.score {
                return Some(Ordering::Less);
            }
            return Some(Ordering::Greater);
        }
        // expected: M-1 M-2 M-3 ... M3 M2 M1
        if self.mate_in < 0 {
            if other.mate_in < 0 {
                if self.mate_in < other.mate_in {
                    return Some(Ordering::Greater);
                }
                return Some(Ordering::Less);
            }
            return Some(Ordering::Less);
        }
        if other.mate_in < 0 {
            return Some(Ordering::Greater);
        }
        if self.mate_in < other.mate_in {
            return Some(Ordering::Greater);
        }
        Some(Ordering::Less)
    }
}

impl Ord for Eval {
    fn cmp(&self, other: &Self) -> Ordering {
        // expected: -3 -2 -1 ... 1 2 3
        if self.mate_in == other.mate_in {
            if self.score == other.score {
                return Ordering::Equal;
            }
            if self.score < other.score {
                return Ordering::Less;
            }
            return Ordering::Greater;
        }
        // expected: M-1 M-2 M-3 ... M3 M2 M1
        if self.mate_in < 0 {
            if other.mate_in < 0 {
                if self.mate_in < other.mate_in {
                    return Ordering::Greater;
                }
                return Ordering::Less;
            }
            return Ordering::Less;
        }
        if other.mate_in < 0 {
            return Ordering::Greater;
        }
        if self.mate_in < other.mate_in {
            return Ordering::Greater;
        }
        Ordering::Less
    }
}

// need to store move eval with the move itself
pub struct EvalMov {
    pub mov: Mov,
    pub eval: Eval
}