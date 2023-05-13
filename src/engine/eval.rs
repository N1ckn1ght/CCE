use std::cmp::Ordering;
use crate::board::mov::Mov;

// if there's a winning sequence, we should look onto the fastest possible, not on the score
#[derive(Clone, Copy)]
pub struct Eval {
    pub score: f32,
    pub mate_in: i8
}

impl Eval {
    pub const BIG_SCORE: f32 = 1048576.0;
    pub const BIG_MATE: i8 = 127;

    pub fn new(score: f32, mate_in: i8) -> Self {
        Eval { score, mate_in }
    }

    pub fn equal() -> Eval {
        Eval { score: 0.0, mate_in: 0 }
    }

    pub fn highest() -> Eval {
        Eval { score: Eval::BIG_SCORE, mate_in: 1 }
    }

    pub fn lowest() -> Eval {
        Eval { score: -Eval::BIG_SCORE, mate_in: -1 }
    }

    pub fn higher() -> Eval {
        Eval { score: -Eval::BIG_SCORE, mate_in: 1 }
    }

    pub fn lower() -> Eval {
        Eval { score: Eval::BIG_SCORE, mate_in: -1 }
    }

    pub fn high() -> Eval {
        Eval { score: -Eval::BIG_SCORE, mate_in: Eval::BIG_MATE }
    }

    pub fn low() -> Eval {
        Eval { score: Eval::BIG_SCORE, mate_in: -Eval::BIG_MATE }
    }
}

impl PartialEq for Eval {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score && self.mate_in == other.mate_in
    }
}

impl Eq for Eval {}

impl Ord for Eval {
    fn cmp(&self, other: &Self) -> Ordering {
        // expected: -3 -2 -1 ... 1 2 3
        if self.mate_in == other.mate_in {
            return self.score.total_cmp(&other.score);
        }
        // expected: M-3 M-2 M-1 ... M1 M2 M3
        if self.mate_in > 0 && other.mate_in > 0 {
            return self.mate_in.cmp(&other.mate_in).reverse().then(Ordering::Less)
        }
        if self.mate_in < 0 && other.mate_in < 0 {
            return self.mate_in.cmp(&other.mate_in).reverse().then(Ordering::Greater)
        }
        self.mate_in.signum().cmp(&other.mate_in.signum()) // .then(Ordering::Equal)
    }
}

impl PartialOrd for Eval {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// need to store move eval with the move itself
#[derive(Clone, Copy)]
pub struct EvalMov {
    pub mov: Mov,
    pub eval: Eval
}

// Sum: 64 + 8 + 8 + 8 = 88 bit (per hashed position)
#[derive(Clone, Copy)]
pub struct EvalHashed {
    pub eval: Eval,
    pub depth: i8,
    pub iter: u8,
    pub playcount: u8,
    pub evaluated: bool
}

impl EvalHashed {
    pub fn new(iter: u8) -> Self {
        Self { eval: Eval::equal(), depth: 0, iter, playcount: 1, evaluated: false } 
    }

    pub fn evaluated(eval: Eval, depth: i8, iter: u8, playcount: u8) -> Self {
        Self { eval, depth, iter, playcount, evaluated: true }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::{min, max};
    
    // Eval comparator tests

    #[test]
    fn test_eval_basic_cmp_01() {
        let a = Eval { score: 0.0, mate_in: 0 };
        let b = Eval { score: 0.0, mate_in: 1 };
        assert_eq!(a < b, true);
    }

    #[test]
    fn test_eval_basic_cmp_02() {
        let a = Eval { score: 0.0, mate_in: 0 };
        let c = Eval { score: 0.0, mate_in: -1};
        assert_eq!(a > c, true);
    }

    #[test]
    fn test_eval_basic_cmp_03() {
        let a = Eval { score: 0.0, mate_in: 0 };
        let d = Eval { score: 0.0, mate_in: -2};
        assert_eq!(a > d, true);
    }

    #[test]
    fn test_eval_basic_cmp_04() {
        let b = Eval { score: 0.0, mate_in: 1 };
        let d = Eval { score: 0.0, mate_in: -2};
        assert_eq!(b > d, true);
    }

    #[test]
    fn test_eval_basic_cmp_05() {
        let c = Eval { score: 0.0, mate_in: -1};
        let d = Eval { score: 0.0, mate_in: -2};
        assert_eq!(c < d, true);
    }

    #[test]
    fn test_eval_basic_cmp_06() {
        let a = Eval { score: 0.0, mate_in: 0 };
        assert_eq!(a == a, true);
    }

    #[test]
    fn test_eval_basic_cmp_07() {
        let a = Eval { score: 0.0, mate_in: 0 };
        let e = Eval { score: 1.0, mate_in: 0 };
        assert_eq!(a < e, true);
    }

    #[test]
    fn test_eval_basic_cmp_08() {
        let a = Eval { score: 0.0, mate_in: 0 };
        let f = Eval { score: -1., mate_in: 0 };
        assert_eq!(a > f, true);
    }

    #[test]
    fn test_eval_basic_cmp_09() {
        let a = Eval { score: 0.0, mate_in: 0 };
        let g = Eval { score: 1.0, mate_in: -1};
        assert_eq!(a > g, true);
    }

    #[test]
    fn test_eval_basic_cmp_10() {
        let g = Eval { score: 1.0, mate_in: -1};
        assert_eq!(g == g, true);
    }

    #[test]
    fn test_eval_basic_cmp_11() {
        let c = Eval { score: 0.0, mate_in: -1};
        let g = Eval { score: 1.0, mate_in: -1};
        assert_eq!(g > c, true);
    }

    #[test]
    fn test_eval_std_cmp_01() {
        let a = Eval { score: 1.0, mate_in: 1};
        let b = Eval { score: 0.0, mate_in: 0};
        assert_eq!(min(a, b) == b, true);
    }

    #[test]
    fn test_eval_std_cmp_02() {
        let a = Eval { score: 1.0, mate_in: 1};
        let b = Eval { score: 0.0, mate_in: 0};
        assert_eq!(min(b, a) == b, true);
    }

    #[test]
    fn test_eval_std_cmp_03() {
        let a = Eval { score: 1.0, mate_in: 1};
        let b = Eval { score: 0.0, mate_in: 0};
        assert_eq!(max(a, b) == a, true);
    }

    #[test]
    fn test_eval_std_cmp_04() {
        let c = Eval { score: 0.0, mate_in: 1};
        let d = Eval { score: 0.0, mate_in: -1};
        assert_eq!(max(c, d) == c, true);
    }

    #[test]
    fn test_eval_std_cmp_05() {
        let c = Eval { score: 0.0, mate_in: 1};
        let e = Eval { score: 0.0, mate_in: 2};
        assert_eq!(max(c, e) == c, true);
    }

    #[test]
    fn test_eval_std_cmp_07() {
        let f = Eval { score: 10.0, mate_in: 2};
        let g = Eval { score: 9.0, mate_in: 1};
        assert_eq!(max(f, g) == g, true);
    }

    #[test]
    fn test_eval_std_cmp_08() {
        let f = Eval { score: 17.0, mate_in: 2};
        let g = Eval { score: 9.0, mate_in: 1};
        assert_eq!(min(f, g) == f, true);
    }

    #[test]
    fn test_eval_std_cmp_09() {
        let f = Eval { score: 10.0, mate_in: -2};
        let g = Eval { score: 9.0, mate_in: -1};
        assert_eq!(max(f, g) == f, true);
    }

    #[test]
    fn test_eval_std_cmp_10() {
        let f = Eval { score: -17.0, mate_in: -2};
        let g = Eval { score: -9.0, mate_in: -1};
        assert_eq!(min(f, g) == g, true);
    }

    #[test]
    fn test_eval_advanced_01() {
        let a = Eval { score: 15., mate_in: 0};
        let b = Eval { score: -15., mate_in: 0 };
        let c = Eval { score: 0., mate_in: 16 };
        let d = Eval { score: 0., mate_in: -16 };
        assert_eq!(a > b, true);
        assert_eq!(a < c, true);
        assert_eq!(a > d, true);
        assert_eq!(b < c, true);
        assert_eq!(b > d, true);
        assert_eq!(c > d, true);
    }

    #[test]
    fn test_eval_advanced_02() {
        let a = Eval { score: 15., mate_in: 0};
        let b = Eval { score: -15., mate_in: 0 };
        let c = Eval { score: 0., mate_in: 16 };
        let d = Eval { score: 0., mate_in: -16 };
        assert_eq!(a == a, true);
        assert_eq!(b == b, true);
        assert_eq!(c == c, true);
        assert_eq!(d == d, true);
    }

    #[test]
    fn test_eval_advanced_03() {
        let a = Eval { score: 10., mate_in: 0 };
        let b = Eval { score: 9.0, mate_in: 1 };
        assert_eq!(max(a, b) == b, true);
        assert_eq!(min(a, b) == b, false);
        assert_eq!(a > b, false);
        assert_eq!(a < b, true);
        assert_eq!(a >= b, false);
        assert_eq!(a <= b, true);
    }
}