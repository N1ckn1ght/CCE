#[cfg(test)]
mod tests {
    use std::cmp::{min, max};
    use crate::{engine::{eval::Eval, minimax::Minimax}, board::board::Board, characters::materialist::Materialist, utils::utils::move_to_user};

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


    // Mate-in-X moves minimax engine test
    // Default "Materialist" character will be used

    #[test]
    fn test_minimax_find_mate_01() {
        let mut b = Board::parse_fen("5k2/5ppp/5PPP/8/8/8/4R3/4R1K1 w - - 0 1".to_string());
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 6);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "e2e8".to_string(), true);
        assert_eq!(moves[0].eval.mate_in == 1, true);
    }
}