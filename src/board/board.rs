use crate::board::coord::Coord;
use crate::board::mov::Mov;
use phf::phf_map;
use std::char;
use std::cmp::{max, min};
use std::vec::Vec;

pub static PIECES: phf::Map<char, u8> = phf_map! {
    ' ' => 0,
    'p' => 2,
    'P' => 3,
    'k' => 4,
    'K' => 5,
    'n' => 6,
    'N' => 7,
    'b' => 8,
    'B' => 9,
    'r' => 10,
    'R' => 11,
    'q' => 12,
    'Q' => 13
};

pub enum Check {
    NotInCheck,
    InCheck,
    InDoubleCheck,
    InMate
}

#[derive(Clone)]
pub struct Board {
    // FEN information

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
    no: u16,

    // Additional information that's necessary in order to speedup the search of legal moves
    white_king_location: Coord,
    black_king_location: Coord
}

impl Board {
    pub fn new() -> Board {
        Board{
            field: get_default_board(),
            history: Vec::new(),
            white_to_move: true,
            en_passant: Coord{y: 8, x : 8},
            castling: 240,
            hmw: 0,
            no: 1,
            white_king_location: Coord{y: 0, x : 4},
            black_king_location: Coord{y: 7, x : 4}
        }
    }

    pub fn parse_fen(FEN: String) -> Board {
        let mut field: [[u8; 8]; 8] = [[0; 8]; 8];
        let mut history: Vec<Mov> = Vec::new();
        let mut white_to_move: bool = true;
        let mut en_passant: Coord = Coord{y: 8, x : 8};
        let mut castling: u8 = 0;
        let mut hmw: u8 = 0;
        let mut no: u16 = 1;
        let mut white_king_location = Coord{y: 8, x : 8};
        let mut black_king_location = Coord{y: 8, x : 8};

        let parts = FEN.split(" ");
        let mut col: usize = 0;
        let mut row: usize = 7;
        let mut pn: u8 = 0;
        // since split returns lazy iterator...
        // TODO: find a better way to parse a string with spaces :/
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
                        field[row][col] = piece_to_value(c);

                        // Addon
                        if c == 'k' {
                            black_king_location.y = row;
                            black_king_location.x = col
                        } else if c == 'K' {
                            white_king_location.y = row;
                            white_king_location.x = col
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
                        en_passant.y = row;
                        en_passant.x = col;
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

        Board{field, history, white_to_move, en_passant, castling, hmw, no, white_king_location, black_king_location}
    }

    pub fn get_legal_moves(& self, check_status: Option<Check>) -> Vec<Mov> {
        let check: Check = check_status.unwrap_or(Check::InCheck);
        let color_bit: u8 = self.white_to_move as u8;
        match check {
            Check::NotInCheck => {
                // scan for any moves
            },
            Check::InCheck => {
                // scan for any moves carefully
                for y in 0..7 {
                    for x in 0..7 {
                        if self.field[y][x] > 0 {
                            let piece = self.field[y][x] - color_bit;
                            if piece == PIECES[&'p'] {

                            } else if piece == PIECES[&'k'] {

                            } else if piece == PIECES[&'n'] {

                            } else if piece == PIECES[&'b'] {
                                // search diag
                            } else if piece == PIECES[&'r'] {
                                // search straight
                            } else if piece == PIECES[&'q'] {
                                // search diag & straight
                            }
                        }
                    }
                }
            },
            Check::InDoubleCheck => {
                // scan for only king moves
            },
            Check::InMate => {
                return Vec::new()
            }
        }
        Vec::new()
    }

    // debug output for now
    pub fn print(& self) {
        for i in 0..8 {
            for j in 0..8 {
                print!("{}\t", self.field[7 - i][j]);
            }
            println!();
        }
        println!();
    }

    pub fn get_current_king_coord(& self, is_current_color: bool) -> &Coord {
        // if white_to_move and current_color, we are searching for black pieces attacking white king
        let color = is_current_color ^ self.white_to_move;
        if color {
            &self.black_king_location
        } else {
            &self.white_king_location
        }
    }

    // this is rather slow and should not be extensive used
    // if color is WHITE, we are searching for WHITE threats for a BLACK piece
    // 1 stands for WHITE, 0 stands for BLACK
    pub fn is_under_attack(& self, y: usize, x: usize, color_of_attacker: bool, checks: [bool; 5]) -> bool {
        let color_bit = color_of_attacker as u8;
        (checks[0] && self.is_under_attack_bq(y, x, color_bit)) || 
        (checks[1] && self.is_under_attack_rq(y, x, color_bit)) ||
        (checks[2] && self.is_under_attack_n(y, x, color_bit)) || 
        (checks[3] && self.is_under_attack_k(y, x, color_of_attacker)) ||
        (checks[4] && self.is_under_attack_p(y, x, color_of_attacker))
    }

    // check if opponent's knight is attacking this cell
    fn is_under_attack_n(& self, y: usize, x: usize, color_bit: u8) -> bool {
        for i in 1..2 {
            if in_bound(y + 3, x + i, i, 0) && self.field[y + 3 - i][x + i] == PIECES[&'n'] + color_bit {
                return true;
            }
            if in_bound(y, x + 3, i, i) && self.field[y - i][x + 3 - i] == PIECES[&'n'] + color_bit {
                return true;
            }
            if in_bound(y + i, x, 3, i) && self.field[y + i - 3][x - i] == PIECES[&'n'] + color_bit {
                return true;
            }
            if in_bound(y + i, x + i, 0, 3) && self.field[y + i][x + i - 3] == PIECES[&'n'] + color_bit {
                return true;
            }
        }
        false
    }

    // check if opponent's biship or queen is attacking this cell by diagonal
    fn is_under_attack_bq(& self, y: usize, x: usize, color_bit: u8) -> bool {
        let mut i: usize = 1;
        let mut piece: u8;
        while in_bound(y, x, i, i) {
            piece = self.field[y - i][x - i];
            i += 1;
            if piece > 0 {
                piece -= color_bit;
            } else {
                continue;
            }
            if piece == PIECES[&'b'] || piece == PIECES[&'q'] {
                return true;
            } else {
                break;
            }
        }
        i = 1;
        while in_bound(y, x + i, i, 0) {
            piece = self.field[y - i][x + i];
            i += 1;
            if piece > 0 {
                piece -= color_bit;
            } else {
                continue;
            }
            if piece == PIECES[&'b'] || piece == PIECES[&'q'] {
                return true;
            } else {
                break;
            }
        }
        i = 1;
        while in_bound(y + i, x + i, 0, 0) {
            piece = self.field[y + i][x + i];
            i += 1;
            if piece > 0 {
                piece -= color_bit;
            } else {
                continue;
            }
            if piece == PIECES[&'b'] || piece == PIECES[&'q'] {
                return true;
            } else {
                break;
            }
        }
        i = 1;
        while in_bound(y + i, x, 0, i) {
            piece = self.field[y + i][x - i];
            i += 1;
            if piece > 0 {
                piece -= color_bit;
            } else {
                continue;
            }
            if piece == PIECES[&'b'] || piece == PIECES[&'q'] {
                return true;
            } else {
                break;
            }
        }
        false
    }

    // check if opponent's rook or queen is attacking this cell by horizontal or vertical
    fn is_under_attack_rq(& self, y: usize, x: usize, color_bit: u8) -> bool {
        let mut i: usize = 1;
        let mut piece: u8;
        while in_bound(y, x, i, 0) {
            piece = self.field[y - i][x];
            i += 1;
            if piece > 0 {
                piece -= color_bit;
            } else {
                continue;
            }
            if piece == PIECES[&'r'] || piece == PIECES[&'q'] {
                return true;
            } else {
                break;
            }
        }
        i = 1;
        while in_bound(y + i, x, 0, 0) {
            piece = self.field[y + i][x];
            i += 1;
            if piece > 0 {
                piece -= color_bit;
            } else {
                continue;
            }
            if piece == PIECES[&'r'] || piece == PIECES[&'q'] {
                return true;
            } else {
                break;
            }
        }
        i = 1;
        while in_bound(y, x + i, 0, 0) {
            piece = self.field[y][x + i];
            i += 1;
            if piece > 0 {
                piece -= color_bit;
            } else {
                continue;
            }
            if piece == PIECES[&'r'] || piece == PIECES[&'q'] {
                return true;
            } else {
                break;
            }
        }
        i = 1;
        while in_bound(y, x, 0, i) {
            piece = self.field[y][x - i];
            i += 1;
            if piece > 0 {
                piece -= color_bit;
            } else {
                continue;
            }
            if piece == PIECES[&'r'] || piece == PIECES[&'q'] {
                return true;
            } else {
                break;
            }
        }
        false
    }

    // check if opponent's king is attacking this cell
    fn is_under_attack_k(& self, y: usize, x: usize, color_of_attacker: bool) -> bool {
        let king: &Coord;
        if color_of_attacker {
            king = &self.black_king_location;
        } else {
            king = &self.white_king_location;
        }
        if max(king.y, y) - min(king.y, y) < 2 && max(king.x, x) - min(king.x, x) < 2 {
            return true;
        }
        false
    }

    // check if opponent's pawn is attacking this cell
    fn is_under_attack_p(& self, y: usize, x: usize, color_of_attacker: bool) -> bool {
        if color_of_attacker {
            if in_bound(y + 1, x + 1, 0, 0) && self.field[y + 1][x + 1] == PIECES[&'P'] {
                return true;
            }
            if in_bound(y + 1, x, 0, 1) && self.field[y + 1][x - 1] == PIECES[&'P'] {
                return true;
            }
        } else {
            if in_bound(y, x + 1, 1, 0) && self.field[y - 1][x + 1] == PIECES[&'p'] {
                return true;
            }
            if in_bound(y, x, 1, 1) && self.field[y - 1][x - 1] == PIECES[&'p'] {
                return true;
            }
        }
        // check for en_passant, though this is a very specific case that may never be used
        self.en_passant.y == y && self.en_passant.x == x
    }

    // if that square is empty or has a piece of a color_bit color, this move will be added
    fn add_legal_moves_n(& self, vec: &mut Vec<Mov>, y: usize, x: usize, color_bit: u8) {
        for i in 1..2 {
            if in_bound(y + 3, x + i, i, 0) {
                let coord = Coord{y: y + 3 - i, x: x + i};
                let piece = self.field[coord.y][coord.x];
                if piece == 0 {
                    vec.push(Mov{data: 0, from: Coord{y, x}, to: coord});
                } else if piece & 1 != color_bit {
                    vec.push(Mov{data: piece + 32, from: Coord{y, x}, to: coord});
                }
            }
            if in_bound(y, x + 3, i, i) {
                let coord = Coord{y: y - i, x: x + 3 - i};
                let piece = self.field[coord.y][coord.x];
                if piece == 0 {
                    vec.push(Mov{data: 0, from: Coord{y, x}, to: coord});
                } else if piece & 1 != color_bit {
                    vec.push(Mov{data: piece + 32, from: Coord{y, x}, to: coord});
                }
            }
            if in_bound(y + i, x, 3, i) {
                let coord = Coord{y: y + i - 3, x: x - i};
                let piece = self.field[coord.y][coord.x];
                if piece == 0 {
                    vec.push(Mov{data: 0, from: Coord{y, x}, to: coord});
                } else if piece & 1 != color_bit {
                    vec.push(Mov{data: piece + 32, from: Coord{y, x}, to: coord});
                }
            }
            if in_bound(y + i, x + i, 0, 3) {
                let coord = Coord{y: y + i, x: x + i - 3};
                let piece = self.field[coord.y][coord.x];
                if piece == 0 {
                    vec.push(Mov{data: 0, from: Coord{y, x}, to: coord});
                } else if piece & 1 != color_bit {
                    vec.push(Mov{data: piece + 32, from: Coord{y, x}, to: coord});
                }
            }
        }
    }

    fn add_legal_moves_bq(& self, vec: &mut Vec<Mov>, y: usize, x: usize, color_bit: u8) {
        let mut i: usize = 1;
        let mut piece: u8;
        while in_bound(y, x, i, i) {
            let piece = self.field[y - i][y - x];
            if piece == 0 {
                vec.push(Mov{data: 0, from: Coord{y, x}, to: Coord{y: y - 1, x: x - 1}});
            } else if piece & 1 != color_bit {
                vec.push(Mov{data: piece + 32, from: Coord{y, x}, to: Coord{y: y - 1, x: x - 1}})
            } else {
                break;
            }
        //         if piece == 0 {
        //             vec.push(Mov{data: 0, from: Coord{y, x}, to: coord});
        //         } else if piece & 1 != color_bit {
        //             vec.push(Mov{data: piece + 32, from: Coord{y, x}, to: coord});
        //         }
            piece = self.field[y - i][x - i];
            i += 1;
            if piece > 0 {
                piece -= color_bit;
            } else {
                continue;
            }
            if piece == PIECES[&'b'] || piece == PIECES[&'q'] {
                return true;
            } else {
                break;
            }
        }
        i = 1;
        while in_bound(y, x + i, i, 0) {
            piece = self.field[y - i][x + i];
            i += 1;
            if piece > 0 {
                piece -= color_bit;
            } else {
                continue;
            }
            if piece == PIECES[&'b'] || piece == PIECES[&'q'] {
                return true;
            } else {
                break;
            }
        }
        i = 1;
        while in_bound(y + i, x + i, 0, 0) {
            piece = self.field[y + i][x + i];
            i += 1;
            if piece > 0 {
                piece -= color_bit;
            } else {
                continue;
            }
            if piece == PIECES[&'b'] || piece == PIECES[&'q'] {
                return true;
            } else {
                break;
            }
        }
        i = 1;
        while in_bound(y + i, x, 0, i) {
            piece = self.field[y + i][x - i];
            i += 1;
            if piece > 0 {
                piece -= color_bit;
            } else {
                continue;
            }
            if piece == PIECES[&'b'] || piece == PIECES[&'q'] {
                return true;
            } else {
                break;
            }
        }
        // for i in 1..2 {
        //     if in_bound(y + 3, x + i, i, 0) {
        //         let coord = Coord{y: y + 3 - i, x: x + i};
        //         let piece = self.field[coord.y][coord.x];
        //         if piece == 0 {
        //             vec.push(Mov{data: 0, from: Coord{y, x}, to: coord});
        //         } else if piece & 1 != color_bit {
        //             vec.push(Mov{data: piece + 32, from: Coord{y, x}, to: coord});
        //         }
        //     }
        //     if in_bound(y, x + 3, i, i) {
        //         let coord = Coord{y: y - i, x: x + 3 - i};
        //         let piece = self.field[coord.y][coord.x];
        //         if piece == 0 {
        //             vec.push(Mov{data: 0, from: Coord{y, x}, to: coord});
        //         } else if piece & 1 != color_bit {
        //             vec.push(Mov{data: piece + 32, from: Coord{y, x}, to: coord});
        //         }
        //     }
        //     if in_bound(y + i, x, 3, i) {
        //         let coord = Coord{y: y + i - 3, x: x - i};
        //         let piece = self.field[coord.y][coord.x];
        //         if piece == 0 {
        //             vec.push(Mov{data: 0, from: Coord{y, x}, to: coord});
        //         } else if piece & 1 != color_bit {
        //             vec.push(Mov{data: piece + 32, from: Coord{y, x}, to: coord});
        //         }
        //     }
        //     if in_bound(y + i, x + i, 0, 3) {
        //         let coord = Coord{y: y + i, x: x + i - 3};
        //         let piece = self.field[coord.y][coord.x];
        //         if piece == 0 {
        //             vec.push(Mov{data: 0, from: Coord{y, x}, to: coord});
        //         } else if piece & 1 != color_bit {
        //             vec.push(Mov{data: piece + 32, from: Coord{y, x}, to: coord});
        //         }
        //     }
        // }
    }
}

pub fn in_bound(y: usize, x: usize, y_sub: usize, x_sub: usize) -> bool {
    if y > 7 + y_sub || x > 7 + x_sub || y_sub > y || x_sub > x {
        return false;
    }
    true
}

fn get_default_board() -> [[u8; 8]; 8] {
    let mut field = [[0; 8]; 8];
    for i in 0..7 {
        field[1][i] = PIECES[&'K'];
        field[6][i] = PIECES[&'k'];
    }
    field[0][0] = PIECES[&'R'];
    field[0][1] = PIECES[&'N'];
    field[0][2] = PIECES[&'B'];
    field[0][3] = PIECES[&'Q'];
    field[0][4] = PIECES[&'K'];
    field[0][5] = PIECES[&'B'];
    field[0][6] = PIECES[&'N'];
    field[0][7] = PIECES[&'R'];
    field[7][0] = PIECES[&'r'];
    field[7][1] = PIECES[&'n'];
    field[7][2] = PIECES[&'b'];
    field[7][3] = PIECES[&'q'];
    field[7][4] = PIECES[&'k'];
    field[7][5] = PIECES[&'b'];
    field[7][6] = PIECES[&'n'];
    field[7][7] = PIECES[&'r'];
    field
}

fn piece_to_value(piece: char) -> u8 {
    PIECES[&piece]
}

fn value_to_piece(value: u8) -> char {
    for (key, value_) in &PIECES {
        if value == *value_ {
            return *key
        }
    }
    panic!("No such piece with number: \"{}\"", value)
}