use bimap::BiMap;
use std::char;

pub struct Bimaps {
    pub pieces: Bimap,
    pub castles: Bimap,
    pub promotions: Bimap
}

impl Bimaps {
    pub fn init() -> Bimaps {
        let mut pieces = Bimap::<char, u8>::new();
        let mut castles = Bimap::<char, u8>::new();
        let mut promotions = Bimap::<char, u8>::new();

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
        casltes.insert('k', 32);
        castles.insert('Q', 64);
        casltes.insert('K', 128);

        promotions.insert('b', 0);
        promotions.insert('r', 2);
        promotions.insert('n', 4);
        promotions.insert('q', 6);

        Bimaps{pieces, castles, promotions}
    }
}