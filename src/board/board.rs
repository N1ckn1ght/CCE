use std::vec::Vec;
use crate::board::mov::Mov;
use crate::board::coord::Coord;

#[derive(Clone)]
pub struct Board {
    // 12 - queen, 10 - rook, 8 - bishop, 6 - knight, 4 - king, 2 - pawn
    // +1 if it's a white piece
    // this choice is for better synergy with mov structure
    // white will have their pieces on 0, 1 horizontals, black on 6, 7
    pub field: [[u8; 8]; 8],
    // move storage for a takeback function
    history: Vec<Mov>,
    // 1 - white to move, 0 - black to move 
    white_to_move: bool,
    // coordinate of en passant if possible, otherwise 8, 8
    en_passant: Coord,
    // castling possibility, 1-2 bits are for white O-O and O-O-O, 3-4 for black
    castling: u8,
    // half-moves counter since last capture or pawn move
    hmw: u8,
    // move number, shall be incremented after every black move
    // more safe to use 2 bytes since it's proven possible to have a game with 300 moves or more
    no: u16
}

impl Board {
    pub fn parse_fen(FEN: String) -> Board {
        let mut field: [[u8; 8]; 8] = [[0; 8]; 8];
        let mut history: Vec<Mov> = Vec::new();
        let mut white_to_move: bool = true;
        let mut en_passant: Coord = Coord::new(8, 8);
        let mut castling: u8 = 0;
        let mut hmw: u8 = 0;
        let mut no: u16 = 1;

        let parts = FEN.split(" ");
        let mut col: usize = 0;
        let mut row: usize = 7;
        let mut pn: u8 = 0;
        // since split returns lazy iterator...
        // TODO: find a better way
        for part in parts {
            if pn == 0 {
                for c in part.chars() {
                    if c >= '1' && c <= '8' {
                        col += c as usize - '0' as usize;
                    } else if c == '/' {
                        if row > 0 {
                            row -= 1;
                        }
                        col = 0;
                    } else {
                        match c {
                            'p' => field[row][col] = 2,
                            'k' => field[row][col] = 4,
                            'n' => field[row][col] = 6,
                            'b' => field[row][col] = 8,
                            'r' => field[row][col] = 10,
                            'q' => field[row][col] = 12,
                            'P' => field[row][col] = 3,
                            'K' => field[row][col] = 5,
                            'N' => field[row][col] = 7,
                            'B' => field[row][col] = 9,
                            'R' => field[row][col] = 11,
                            'Q' => field[row][col] = 13,
                            // TODO: more detailed panic! messages
                            _ => panic!("Impossible FEN, cannot import")
                        }
                        col += 1;  
                    }
                }
                pn = 1;
            } else if pn == 1 {
                for c in part.chars() {
                    if c == 'b' {
                        white_to_move = false;
                    }
                }
                pn = 2;
            } else if pn == 2 {
                for c in part.chars() {
                    match c {
                        'K' => castling += 128,
                        'Q' => castling += 64,
                        'k' => castling += 32,
                        'q' => castling += 16,
                        '-' => (),
                        _ => panic!("Impossible FEN, cannot import.")
                    }
                }
                pn = 3;
            } else if pn == 3 {
                col = 8;
                row = 8;
                let mut pn2: u8 = 0;
                for c in part.chars() {
                    if pn2 == 0 {
                        if c == '-' {
                            break;
                        }
                        row = c as usize - 'a' as usize;
                        pn2 = 1;
                    } else {
                        col = c as usize - '0' as usize;
                        en_passant.y = row as u8;
                        en_passant.x = col as u8;
                    }
                }
                pn = 4;
            } else if pn == 4 {
                hmw = part.parse().unwrap();
                pn = 5;
            } else {
                no = part.parse().unwrap();
                break;
            }
        }
        Board{field, history, white_to_move, en_passant, castling, hmw, no}
    }

    pub fn new() -> Board {
        let mut field: [[u8; 8]; 8] = get_default_board();
        let mut history: Vec<Mov> = Vec::new();
        let mut white_to_move: bool = true;
        let mut en_passant: Coord = Coord::new(8, 8);
        let mut castling: u8 = 240;
        let mut hmw: u8 = 0;
        let mut no: u16 = 1;
        Board{field, history, white_to_move, en_passant, castling, hmw, no}
    }

    pub fn print(& self) {
        for i in 0..8 {
            for j in 0..8 {
                print!("{}\t", self.field[7 - i][j]);
            }
            println!();
        }
        println!();
    }
}

fn get_default_board() -> [[u8; 8]; 8] {
    let mut field = [[0; 8]; 8];
    for i in 0..7 {
        field[1][i] = 3;
        field[6][i] = 2;
    }
    field[0][0] = 11;
    field[0][1] = 7;
    field[0][2] = 9;
    field[0][3] = 13;
    field[0][4] = 5;
    field[0][5] = 9;
    field[0][6] = 7;
    field[0][7] = 11;
    field[7][0] = 10;
    field[7][1] = 6;
    field[7][2] = 8;
    field[7][3] = 12;
    field[7][4] = 4;
    field[7][5] = 8;
    field[7][6] = 6;
    field[7][7] = 10;
    field
}