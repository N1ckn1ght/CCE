use bimap::BiMap;
use std::char;

#[derive(Clone)]
pub struct Bimaps {
    // Mov data will store bits as follows:
    // 1 bit - is this move a double check? | if both - this is checkmate
    // 2 bit - is this move a check?        |
    // 3 bits - 5 bits: captured piece, if any
    // 110 - queen, 101 - rook, 100 - bishop, 011 - knight, 001 - pawn
    // 6 bits - 7 bits: promotion piece
    // 11 - queen, 10 - knight, 01 - rook, 00 - bishop
    // 8 bit - is this move one of the follows: castling, en passant, promotion?

    pub pieces: BiMap::<char, u8>,
    pub castles: BiMap::<char, u8>,
    pub promotions: BiMap::<char, u8>,
    
    pub shift_piece: u8,
    pub shift_promotion: u8,
    pub mask_piece: u8,
    pub mask_promotion: u8,

    pub bit_check: u8,
    pub bit_double_check: u8
}

impl Bimaps {
    pub fn init() -> Bimaps {
        let mut pieces = BiMap::<char, u8>::new();
        let mut castles = BiMap::<char, u8>::new();
        let mut promotions = BiMap::<char, u8>::new();

        // bit shift to save a piece value
        let shift_piece: u8 = 2;
        // bit shift to save a promotion avlue
        let shift_promotion: u8 = 1;
        // bit mask to extract a (shifted) piece value
        let mask_piece: u8 = 14;
        // bit mask to extract a (shifted) promotion value
        let mask_promotion: u8 = 3;

        let bit_check: u8 = 64;
        let bit_double_check: u8 = 128;

        pieces.insert(' ', 0);
        pieces.insert('p', 2);
        pieces.insert('P', 3);
        pieces.insert('k', 4);
        pieces.insert('K', 5);
        pieces.insert('n', 6);
        pieces.insert('N', 7);
        pieces.insert('b', 8);
        pieces.insert('B', 9);
        pieces.insert('r', 10);
        pieces.insert('R', 11);
        pieces.insert('q', 12);
        pieces.insert('Q', 13);

        castles.insert('q', 16);
        castles.insert('k', 32);
        castles.insert('Q', 64);
        castles.insert('K', 128);

        promotions.insert('b', 0);
        promotions.insert('r', 1);
        promotions.insert('n', 2);
        promotions.insert('q', 3);

        Bimaps{pieces, castles, promotions, shift_piece, shift_promotion, mask_piece, mask_promotion, bit_check, bit_double_check}
    }
}