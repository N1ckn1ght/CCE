use rand::{rngs::StdRng, SeedableRng, RngCore};
use crate::board::board::Board;

pub struct Hashtable {
    table: [[[u64; 12]; 8]; 8],
    color: u64,
    castlings: [u64; 4],
    en_passant: [[u64; 8]; 2]
}

impl Hashtable {
    pub fn new(seed: u64) -> Hashtable {
        let mut table = [[[0; 12]; 8]; 8];
        let mut rng = StdRng::seed_from_u64(seed);
        for row in &mut table {
            for col in &mut *row {
                for piece in &mut *col {
                    *piece = rng.next_u64();
                }
            }
        }
        let color = rng.next_u64();
        let mut castlings = [0; 4];
        for castle in &mut castlings {
            *castle = rng.next_u64();
        }
        let mut en_passant = [[0; 8]; 2];
        for row in &mut en_passant {
            for col in &mut *row {
                *col = rng.next_u64();
            }
        }
        Hashtable { table, color, castlings, en_passant }
    }

    // won't include move counters
    pub fn hash(& self, board: &Board) -> u64 {
        let mut value = 0;
        for i in 0..8 {
            for j in 0..8 {
                let piece = board.field[i][j];
                if piece > 1 {
                    value ^= self.table[i][j][(piece - 2) as usize];
                }
            }
        }
        if board.white_to_move {
            value ^= self.color;
        }
        if board.castling > 0 {
            if board.castling & board.gcl(&'k') > 0 {
                value ^= self.castlings[0];
            }
            if board.castling & board.gcl(&'q') > 0 {
                value ^= self.castlings[0];
            }
            if board.castling & board.gcl(&'K') > 0 {
                value ^= self.castlings[0];
            }
            if board.castling & board.gcl(&'Q') > 0 {
                value ^= self.castlings[0];
            }
        }
        if board.en_passant.y() < 8 {
            if (board.en_passant.y() >> 1) & 1 > 0 {
                value ^= self.en_passant[0][board.en_passant.x() as usize];
            } else {
                value ^= self.en_passant[1][board.en_passant.x() as usize];
            }
        }
        value
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{mov::Mov, coord::Coord};

    // These tests are low on comparisons...

    #[test]
    fn test_hashtable_create_01() {
        let a = Hashtable::new(0);
        let b = Hashtable::new(1);
        let c = Hashtable::new(123456789);
        // private fields
        assert_eq!(a.table[0][0][0] == b.table[0][0][0], false);
        assert_eq!(c.table[7][7][11] == b.table[7][7][11], false);
        assert_eq!(a.table[2][3][4] == a.table[2][3][4], true);
    }

    #[test]
    fn test_hashtable_hashing_01() {
        let a = Hashtable::new(0);
        let b = Board::new();
        let h = a.hash(&b);
        assert_eq!(h == 0, false);
    }

    #[test]
    fn test_hashtable_hashing_02() {
        let a = Hashtable::new(0);
        let b = Board::parse_fen(&"rnbqkbnr/1pp1pppp/p7/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3".to_string());
        let h = a.hash(&b);
        let b2 = Board::parse_fen(&"rnbq1bnr/pppppk1p/8/5p2/4P1pP/5PP1/PPPPN3/RNBQKBR1 b Q h3 0 6".to_string());
        let h2 = a.hash(&b2);
        assert_eq!(h == h2, false);
    }

    #[test]
    fn test_hashtable_hashing_03() {
        let a = Hashtable::new(0);
        let mut b = Board::parse_fen(&"r1bqkb1r/pppp1ppp/2n2n2/4p1N1/2B1P3/8/PPPP1PPP/RNBQK2R b KQkq - 5 4".to_string());
        let h = a.hash(&b);
        b.make_move(&Mov{ data: 0, from: Coord::new(6, 3), to: Coord::new(4, 3)});
        let h2 = a.hash(&b);
        b.revert_move();
        let h3 = a.hash(&b);
        assert_eq!(h == h2, false);
        assert_eq!(h == h3, true);
    }
}