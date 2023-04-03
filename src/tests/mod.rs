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
    // Mate in 1, depth 3, linear;
    fn test_minimax_find_mate_01() {
        let mut b = Board::parse_fen("5k2/5ppp/5PPP/8/8/8/4R3/4R1K1 w - - 0 1".to_string());
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 6);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "e2e8".to_string(), true);
        assert_eq!(moves[0].eval.mate_in == 1, true);
    }

    #[test]
    // Mate in 1, depth 1, promotion;
    fn test_minimax_find_mate_02() {
        let mut b = Board::parse_fen("6k1/4Pppp/5P2/8/8/8/8/6K1 w - - 0 1".to_string());
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 2);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "e7e8q".to_string() || move_to_user(&b, &moves[0].mov) == "e7e8r".to_string(), true);
        assert_eq!(moves[0].eval.mate_in == 1, true);
    }

    #[test]
    // Mate in 2, depth 2, promotion;
    fn test_minimax_find_mate_03() {
        let mut b = Board::parse_fen("6kq/5ppp/4P3/8/8/8/8/BB4K1 w - - 0 1".to_string());
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 4);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "e6e7".to_string(), true);
        assert_eq!(moves[0].eval.mate_in == 2, true);
    }

    #[test]
    // Mate in 3, depth 3, bishop and queen traps the castled king (HARD);
    fn test_minimax_find_mate_04() {
        let mut b = Board::parse_fen("4qrk1/p1r1Bppp/4b3/2p3Q1/8/3P4/PPP2PPP/R3R1K1 w - - 3 19".to_string());
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 6);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "e7f6".to_string(), true);
        assert_eq!(moves[0].eval.mate_in == 3, true);
    }

    #[test]
    // Mate in 2, depth 3, forced Legal Mate sequence;
    fn test_minimax_find_mate_05() {
        let mut b = Board::parse_fen("r2qkbnr/ppp2ppp/2np4/4N3/2B1P3/2N4P/PPPP1PP1/R1BbK2R w KQkq - 0 7".to_string());
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 6);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "c4f7".to_string(), true);
        assert_eq!(moves[0].eval.mate_in == 2, true);
    }

    #[test]
    // Mate in -1, depth 3, almost Fool's mate;
    fn test_minimax_find_mate_06() {
        let mut b = Board::parse_fen("rnbqkbnr/pppp1ppp/8/8/4pPP1/P7/1PPPP2P/RNBQKBNR b KQkq f3 0 3".to_string());
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 6);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "d8h4".to_string(), true);
        assert_eq!(moves[0].eval.mate_in == -1, true);
    }

    #[test]
    // Mate in -3, linear mate, but White is to move and lose;
    fn test_minimax_find_mate_07() {
        let mut b = Board::parse_fen("k3r3/3r4/8/8/8/8/8/5K2 w - - 0 1".to_string());
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 6);
        assert_eq!(moves[0].eval.mate_in == -3, true);
    }

    #[test]
    // Mate in 2, Vukovic Mate #3 (lichess.org), position from Pavol Danek - Stanislav Hanuliak, 2001;
    fn test_minimax_find_mate_08() {
        let mut b = Board::parse_fen("2r5/8/8/5K1k/4N1R1/7P/8/8 w - - 12 67".to_string());
        let mut engine = Minimax::new();
        let moves = engine.eval(&mut b, &Materialist{}, 6);
        assert_eq!(move_to_user(&b, &moves[0].mov) == "e4f6".to_string(), true);
        assert_eq!(moves[0].eval.mate_in == 2, true);
    }
}