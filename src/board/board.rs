use std::char;
use std::cmp::{max, min};
use std::vec::Vec;
use crate::utils::utils::move_to_user;

use super::bimaps::Bimaps;
use super::coord::Coord;
use super::mov::{BoardMov, Mov};

#[derive(PartialEq, Clone, Copy)]
pub enum Check {
    Unknown,
    NotInCheck,
    InCheck,
    InDoubleCheck
}

// question to myself: why pieces couldn't be classes?
#[derive(Clone)]
pub struct Board {
    // white will have their pieces on 0, 1 horizontals, black on 6, 7
    // last bit (0 or 1) is a color bit
    // so if field[i][j] is < 2 then it's an empty square
    // otherwise please rely on board.gpl(&char) or board.gpr(&value) methods
    // chars: 'p', 'n', 'b', 'r', 'q', 'k' (see Bimaps for further reference)
    pub field: [[u8; 8]; 8],
    // move storage for a takeback (revert) function
    history: Vec<BoardMov>,
    // 1 - white to move, 0 - black to move 
    pub white_to_move: bool,
    // coordinate of en passant if possible, otherwise 8, 8
    pub en_passant: Coord,
    // castling possibility, 1-2 bits are for white O-O and O-O-O, 3-4 for black
    pub castling: u8,
    // half-moves counter since last capture or pawn move
    pub hmw: u8,
    // move number, shall be incremented after every black move
    // more safe to use 2 bytes since it's proven possible to have a game with 300 moves or more
    pub no: u16,

    // Additional information that's necessary in order to speedup the search of legal moves
    white_king_location: Coord,
    black_king_location: Coord,

    // TODO: find a better way to store CONSTANT BIMAPS 
    // (they are not constant because Rust says so! shouldn't even be inside struct)
    bimaps: Bimaps

    // TODO: make is_under_attack check more fast when there're less pieces
    // piece_count: [u8; 12]
}

impl Board {
    pub fn new() -> Self {
        let bimaps = Bimaps::init();
        Self {
            field: Board::get_default_board(&bimaps),
            history: Vec::new(),
            white_to_move: true,
            en_passant: Coord::new(8, 8),
            castling: 240,
            hmw: 0,
            no: 1,
            white_king_location: Coord::new(0, 4),
            black_king_location: Coord::new(7, 4),
            bimaps
        }
    }

    pub fn parse_fen(FEN: &str) -> Self {
        let mut field: [[u8; 8]; 8] = [[0; 8]; 8];
        let history: Vec<BoardMov> = Vec::new();
        let mut white_to_move: bool = true;
        let mut en_passant: Coord = Coord::new(8, 8);
        let mut castling: u8 = 0;
        let mut hmw: u8 = 0;
        let mut no: u16 = 1;
        let mut white_king_location = Coord::new(0, 4);
        let mut black_king_location = Coord::new(7, 4);
        let bimaps = Bimaps::init();

        let parts = FEN.split_ascii_whitespace();
        let mut col: u8 = 0;
        let mut row: u8 = 7;
        let mut pn: u8 = 0;
        // since split returns lazy iterator...
        // TODO: find a better way to parse a string with spaces :/
        for part in parts {
            if pn == 0 {
                for c in part.chars() {
                    if ('1'..='8').contains(&c) {
                        col += c as u8 - b'0';
                    } else if c == '/' {
                        row = row.saturating_sub(1);
                        col = 0;
                    } else {
                        field[row as usize][col as usize] = *bimaps.pieces.get_by_left(&c).unwrap();

                        // Addon
                        if c == 'k' {
                            black_king_location.set(row, col);
                        } else if c == 'K' {
                            white_king_location.set(row, col);
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
                        row = c as u8 - b'b';
                        pn2 = 1;
                    } else {
                        col = c as u8 - b'0';
                        en_passant.set(row, col);
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

        Self { field, history, white_to_move, en_passant, castling, hmw, no, white_king_location, black_king_location, bimaps }
    }

    // Careful: this function WILL MAKE A MOVE without additional checks on if it's a legal move or not!
    pub fn make_move(&mut self, mov: &Mov) {
        let piece = self.field[mov.from.y() as usize][mov.from.x() as usize];

        // make a move
        self.history.push(BoardMov{mov: *mov, castling: self.castling, en_passant: self.en_passant, hmw: self.hmw});
        self.field[mov.to.y() as usize][mov.to.x() as usize] = self.field[mov.from.y() as usize][mov.from.x() as usize];
        self.field[mov.from.y() as usize][mov.from.x() as usize] = 0;

        // assuming this move is not capture or a pawn move
        self.hmw += 1;

        let mut temp_en_passant: Coord = Coord::new(8, 8);

        // update king locations + check for special cases (this one is castle)
        if piece == self.gpl(&'k') {
            self.black_king_location.set(mov.to.y(), mov.to.x());
            self.castling &= 192;
            if mov.data & 1 == 1 {
                if mov.to.x() == 6 {
                    // Yep, it should be possible to castle even without the initial rook odd!!
                    self.field[7][5] = self.field[7][7];
                    self.field[7][7] = 0;
                } else {
                    self.field[7][3] = self.field[7][0];
                    self.field[7][0] = 0;
                }
            }
        } else if piece == self.gpl(&'K') {
            self.white_king_location.set(mov.to.y(), mov.to.x());
            self.castling &= 48;
            if mov.data & 1 == 1 {
                if mov.to.x() == 6 {
                    self.field[0][5] = self.field[0][7];
                    self.field[0][7] = 0;
                } else {
                    self.field[0][3] = self.field[0][0];
                    self.field[0][0] = 0;
                }
            }
        } 
        // pawn cases - en passant, pre en passant
        else if piece == self.gpl(&'p') {
            // pawn's move (or capture) - drop hmw
            self.hmw = 0;
            if mov.data & 1 == 1 {
                // promotion or en passant
                if mov.to.y() == 0 {
                    self.field[mov.to.y() as usize][mov.to.x() as usize] = self.rtpv(mov.data);
                } else {
                    self.field[self.en_passant.y() as usize + 1][self.en_passant.x() as usize] = 0;
                }
            } 
            // set en passant if it's two-square move
            else if mov.to.y() + 2 == mov.from.y() {
                temp_en_passant.set(5, mov.from.x());
            }
        } else if piece == self.gpl(&'P') {
            self.hmw = 0;
            if mov.data & 1 == 1 {
                if mov.to.y() == 7 {
                    self.field[mov.to.y() as usize][mov.to.x() as usize] = self.rtpv(mov.data); 
                } else {
                    self.field[self.en_passant.y() as usize - 1][self.en_passant.x() as usize] = 0;
                }
            } else if mov.to.y() == mov.from.y() + 2 {
                temp_en_passant.set(2, mov.from.x());
            }
        }
        // watchout for a rook move that will prevent future castling as well
        else if piece == self.gpl(&'r') && mov.from.y() == 7 {
            if mov.from.x() == 0 {
                self.castling &= 16;
            } else if mov.from.x() == 7 {
                self.castling &= 32;
            }
        } else if piece == self.gpl(&'R') && mov.from.y() == 0 {
            if mov.from.x() == 0 {
                self.castling &= 64;
            } else if mov.from.x() == 7 {
                self.castling &= 128;
            }
        }

        // update/drop counters and next side to move
        if self.ptpv(mov.data) > 1 {
            self.hmw = 0;
        }
        self.white_to_move = !self.white_to_move;
        self.no += self.white_to_move as u16;
        self.en_passant = temp_en_passant;
    }

    pub fn revert_move(&mut self) {
        let bmov: BoardMov = self.history.pop().unwrap();
        let mov: &Mov = &bmov.mov;
        let piece: u8 = self.field[mov.to.y() as usize][mov.to.x() as usize];

        self.field[mov.from.y() as usize][mov.from.x() as usize] = piece;
        self.field[mov.to.y() as usize][mov.to.x() as usize] = self.ptpv(mov.data) + self.white_to_move as u8;
        self.castling = bmov.castling;
        self.en_passant = bmov.en_passant;
        self.hmw = bmov.hmw;

        // reverse castling, revert kings locations
        if piece == self.gpl(&'k') {
            self.black_king_location.set(mov.from.y(), mov.from.x());
            if mov.data & 1 == 1 {
                if mov.to.x() == 6 {
                    self.field[7][7] = self.field[7][5];
                    self.field[7][5] = 0;
                } else {
                    self.field[7][0] = self.field[7][3];
                    self.field[7][3] = 0;
                }
            }
        } else if piece == self.gpl(&'K') {
            self.white_king_location.set(mov.from.y(), mov.from.x());
            if mov.data & 1 == 1 {
                if mov.to.x() == 6 {
                    self.field[0][7] = self.field[0][5];
                    self.field[0][5] = 0;
                } else {
                    self.field[0][0] = self.field[0][3];
                    self.field[0][3] = 0;
                }
            }
        } else if mov.data & 1 == 1 {
            // cancel en passant
            if piece == self.gpl(&'p') + !self.white_to_move as u8 {
                // now it's still other side to move, not takebacken one!
                self.field[(mov.to.y() - 1 + (self.white_to_move as u8) * 2) as usize][mov.to.x() as usize] = self.gpl(&'p') + self.white_to_move as u8;
                // remove duplicated pawn?
                self.field[mov.to.y() as usize][mov.to.x() as usize] = 0;
            } else {
                self.field[mov.from.y() as usize][mov.from.x() as usize] = self.gpl(&'p') + !self.white_to_move as u8;
            }
        }

        self.no -= self.white_to_move as u16;
        self.white_to_move = !self.white_to_move;
    }

    pub fn get_legal_moves(&mut self, current_king_check_status: Option<Check>, save_opponent_king_check_status: Option<bool>) -> Vec<Mov> {
        let mut moves: Vec<Mov> = Vec::default();
        let check: Check = current_king_check_status.unwrap_or(Check::Unknown);
        let color_bit: u8 = self.white_to_move as u8;
        let save: bool = save_opponent_king_check_status.unwrap_or(false);
        
        match check {
            Check::Unknown | Check::InCheck => {
                // scan for any pseudo-legal moves
                for y in 0..8 {
                    for x in 0..8 {
                        if self.field[y as usize][x as usize] > 1 {
                            let piece = self.field[y as usize][x as usize] - color_bit;
                            if piece == self.gpl(&'p') {
                                self.add_legal_moves_p(&mut moves, y, x, color_bit);
                            } else if piece == self.gpl(&'k') {
                                self.add_legal_moves_k(&mut moves, y, x, color_bit, Some(check));
                            } else if piece == self.gpl(&'n') {
                                self.add_legal_moves_n(&mut moves, y, x, color_bit);
                            } else if piece == self.gpl(&'b') {
                                self.add_legal_moves_bq(&mut moves, y, x, color_bit);
                            } else if piece == self.gpl(&'r') {
                                self.add_legal_moves_rq(&mut moves, y, x, color_bit);
                            } else if piece == self.gpl(&'q') {
                                self.add_legal_moves_bq(&mut moves, y, x, color_bit);
                                self.add_legal_moves_rq(&mut moves, y, x, color_bit);
                            }
                        }
                    }
                }
                self.add_legal_moves_en_passant(&mut moves);
                // make careful search if in check for each move for every piece!
                let mut i = 0;
                let mut len = moves.len();
                while i < len {
                    self.make_move(&moves[i]);
                    let current_king: &Coord = self.get_current_king_coord(false);
                    if self.is_under_attack(current_king.y(), current_king.x(), self.white_to_move, [true; 5]) {
                        moves[i] = moves[len - 1];
                        moves.pop();
                        len -= 1;
                    } else {
                        if save {
                            self.add_check_bits(&mut moves[i]);
                        }
                        i += 1;
                    }
                    self.revert_move();
                }
            },
            Check::NotInCheck => {
                // still scan for any pseudo-legal moves
                for y in 0..8 {
                    for x in 0..8 {
                        if self.field[y as usize][x as usize] > 1 {
                            let piece = self.field[y as usize][x as usize] - color_bit;
                            if piece == self.gpl(&'p') {
                                self.add_legal_moves_p(&mut moves, y, x, color_bit);
                            } else if piece == self.gpl(&'k') {
                                self.add_legal_moves_k(&mut moves, y, x, color_bit, Some(Check::NotInCheck));
                            } else if piece == self.gpl(&'n') {
                                self.add_legal_moves_n(&mut moves, y, x, color_bit);
                            } else if piece == self.gpl(&'b') {
                                self.add_legal_moves_bq(&mut moves, y, x, color_bit);
                            } else if piece == self.gpl(&'r') {
                                self.add_legal_moves_rq(&mut moves, y, x, color_bit);
                            } else if piece == self.gpl(&'q') {
                                self.add_legal_moves_bq(&mut moves, y, x, color_bit);
                                self.add_legal_moves_rq(&mut moves, y, x, color_bit);
                            }
                        }
                    }
                }
                self.add_legal_moves_en_passant(&mut moves);
                // make simple search on if in check
                let mut i = 0;
                let mut len = moves.len();
                while i < len {
                    self.make_move(&moves[i]);
                    let current_king: &Coord = self.get_current_king_coord(false);

                    // if it's not a king's move, b/r/q search will be sufficient, but otherwise...
                    let mut checks = [true, true, false, false, false];
                    if self.field[moves[i].to.y() as usize][moves[i].to.x() as usize] == self.gpl(&'k') + !self.white_to_move as u8 {
                        checks[2] = true;
                        checks[3] = true;
                        checks[4] = true;
                    }

                    if self.is_under_attack(current_king.y(), current_king.x(), self.white_to_move, checks) {
                        moves[i] = moves[len - 1];
                        moves.pop();
                        len -= 1;
                    } else {
                        if save {
                            self.add_check_bits(&mut moves[i]);
                        }
                        i += 1;
                    }
                    self.revert_move();
                }
            },
            Check::InDoubleCheck => {
                // now only king can move
                let current_king: Coord = *self.get_current_king_coord(false);
                self.add_legal_moves_k(&mut moves, current_king.y(), current_king.x(), self.white_to_move as u8, Some(Check::InDoubleCheck));
                // make full search on if in check
                let mut i = 0;
                let mut len = moves.len();
                while i < len {
                    self.make_move(&moves[i]);
                    if self.is_under_attack(current_king.y(), current_king.x(), self.white_to_move, [true; 5]) {
                        moves[i] = moves[len - 1];
                        moves.pop();
                        len -= 1;
                    } else {
                        if save {
                            self.add_check_bits(&mut moves[i]);
                        }
                        i += 1;
                    }
                    self.revert_move();
                }
            }
        }

        moves
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
    pub fn is_under_attack(& self, y: u8, x: u8, color_of_attacker: bool, checks: [bool; 5]) -> bool {
        let color_bit = color_of_attacker as u8;

        (checks[0] && self.is_under_attack_bq(y, x, color_bit)) || 
        (checks[1] && self.is_under_attack_rq(y, x, color_bit)) ||
        (checks[2] && self.is_under_attack_n(y, x, color_bit)) || 
        (checks[3] && self.is_under_attack_k(y, x, color_of_attacker)) ||
        (checks[4] && self.is_under_attack_p(y, x, color_of_attacker))
    }

    // check if opponent's knight is attacking this cell
    fn is_under_attack_n(& self, y: u8, x: u8, color_bit: u8) -> bool {
        for i in 1..3 {
            if Self::in_bound(y + 3, x + i, i, 0) && self.field[(y + 3 - i) as usize][(x + i)  as usize] == self.gpl(&'b') + color_bit {
                return true;
            }
            if Self::in_bound(y, x + 3, i, i) && self.field[(y - i) as usize][(x + 3 - i) as usize] == self.gpl(&'n') + color_bit {
                return true;
            }
            if Self::in_bound(y + i, x, 3, i) && self.field[(y + i - 3) as usize][(x - i) as usize] == self.gpl(&'n') + color_bit {
                return true;
            }
            if Self::in_bound(y + i, x + i, 0, 3) && self.field[(y + i) as usize][(x + i - 3) as usize] == self.gpl(&'n') + color_bit {
                return true;
            }
        }
        false
    }

    // check if opponent's biship or queen is attacking this cell by diagonal
    fn is_under_attack_bq(& self, y: u8, x: u8, color_bit: u8) -> bool {
        let mut i: u8 = 1;
        let mut piece: u8;
        while Self::in_bound(y, x, i, i) {
            piece = self.field[(y - i) as usize][(x - i) as usize];
            i += 1;
            if piece > 1 {
                piece -= color_bit;
            } else {
                continue;
            }
            if piece == self.gpl(&'b') || piece == self.gpl(&'q') {
                return true;
            } else {
                break;
            }
        }
        i = 1;
        while Self::in_bound(y, x + i, i, 0) {
            piece = self.field[(y - i) as usize][(x + i) as usize];
            i += 1;
            if piece > 1 {
                piece -= color_bit;
            } else {
                continue;
            }
            if piece == self.gpl(&'b') || piece == self.gpl(&'q') {
                return true;
            } else {
                break;
            }
        }
        i = 1;
        while Self::in_bound(y + i, x + i, 0, 0) {
            piece = self.field[(y + i) as usize][(x + i) as usize];
            i += 1;
            if piece > 1 {
                piece -= color_bit;
            } else {
                continue;
            }
            if piece == self.gpl(&'b') || piece == self.gpl(&'q') {
                return true;
            } else {
                break;
            }
        }
        i = 1;
        while Self::in_bound(y + i, x, 0, i) {
            piece = self.field[(y + i) as usize][(x - i) as usize];
            i += 1;
            if piece > 1 {
                piece -= color_bit;
            } else {
                continue;
            }
            if piece == self.gpl(&'b') || piece == self.gpl(&'q') {
                return true;
            } else {
                break;
            }
        }
        false
    }

    // check if opponent's rook or queen is attacking this cell by horizontal or vertical
    fn is_under_attack_rq(& self, y: u8, x: u8, color_bit: u8) -> bool {
        let mut i: u8 = 1;
        let mut piece: u8;
        while Self::in_bound(y, x, i, 0) {
            piece = self.field[(y - i) as usize][x as usize];
            i += 1;
            if piece > 1 {
                piece -= color_bit;
            } else {
                continue;
            }
            if piece == self.gpl(&'r') || piece == self.gpl(&'q') {
                return true;
            } else {
                break;
            }
        }
        i = 1;
        while Self::in_bound(y + i, x, 0, 0) {
            piece = self.field[(y + i) as usize][x as usize];
            i += 1;
            if piece > 1 {
                piece -= color_bit;
            } else {
                continue;
            }
            if piece == self.gpl(&'r') || piece == self.gpl(&'q') {
                return true;
            } else {
                break;
            }
        }
        i = 1;
        while Self::in_bound(y, x + i, 0, 0) {
            piece = self.field[y as usize][(x + i) as usize];
            i += 1;
            if piece > 1 {
                piece -= color_bit;
            } else {
                continue;
            }
            if piece == self.gpl(&'r') || piece == self.gpl(&'q') {
                return true;
            } else {
                break;
            }
        }
        i = 1;
        while Self::in_bound(y, x, 0, i) {
            piece = self.field[y as usize][(x - i) as usize];
            i += 1;
            if piece > 1 {
                piece -= color_bit;
            } else {
                continue;
            }
            if piece == self.gpl(&'r') || piece == self.gpl(&'q') {
                return true;
            } else {
                break;
            }
        }
        false
    }

    // check if opponent's king is attacking this cell
    fn is_under_attack_k(& self, y: u8, x: u8, color_of_attacker: bool) -> bool {
        let king: &Coord = if color_of_attacker {
            &self.white_king_location
        } else {
            &self.black_king_location
        };
        max(king.y(), y) - min(king.y(), y) < 2 && max(king.x(), x) - min(king.x(), x) < 2
    }

    // check if opponent's pawn is attacking this cell
    fn is_under_attack_p(& self, y: u8, x: u8, color_of_attacker: bool) -> bool {
        if color_of_attacker {
            if Self::in_bound(y, x + 1, 1, 0) && self.field[(y - 1) as usize][(x + 1) as usize] == self.gpl(&'P') {
                return true;
            }
            if Self::in_bound(y, x, 1, 1) && self.field[(y - 1) as usize][(x - 1) as usize] == self.gpl(&'P') {
                return true;
            }
        } else {
            if Self::in_bound(y + 1, x + 1, 0, 0) && self.field[(y + 1) as usize][(x + 1) as usize] == self.gpl(&'p') {
                return true;
            }
            if Self::in_bound(y + 1, x, 0, 1) && self.field[(y + 1) as usize][(x - 1) as usize] == self.gpl(&'p') {
                return true;
            }
        }
        // check for en_passant, though this is a very specific case that may never be used
        self.en_passant.y() == y && self.en_passant.x() == x
    }

    // if possible square for a knight is empty or has a piece of not a color_bit color, this move will be added
    fn add_legal_moves_n(& self, vec: &mut Vec<Mov>, y: u8, x: u8, color_bit: u8) {
        let mut coord: Coord;
        let mut piece: u8;
        for i in 1..3 {
            if Self::in_bound(y + 3, x + i, i, 0) {
                coord = Coord::new(y + 3 - i, x + i);
                piece = self.field[coord.y() as usize][coord.x() as usize];
                if piece < 2 {
                    vec.push(Mov{data: 0, from: Coord::new(y, x), to: coord});
                } else if piece & 1 != color_bit {
                    vec.push(Mov{data: self.psav(piece) , from: Coord::new(y, x), to: coord});
                }
            }
            if Self::in_bound(y, x + 3, i, i) {
                coord = Coord::new(y - i, x + 3 - i);
                piece = self.field[coord.y() as usize][coord.x() as usize];
                if piece < 2 {
                    vec.push(Mov{data: 0, from: Coord::new(y, x), to: coord});
                } else if piece & 1 != color_bit {
                    vec.push(Mov{data: self.psav(piece), from: Coord::new(y, x), to: coord});
                }
            }
            if Self::in_bound(y + i, x, 3, i) {
                coord = Coord::new(y + i - 3, x - i);
                piece = self.field[coord.y() as usize][coord.x() as usize];
                if piece < 2 {
                    vec.push(Mov{data: 0, from: Coord::new(y, x), to: coord});
                } else if piece & 1 != color_bit {
                    vec.push(Mov{data: self.psav(piece), from: Coord::new(y, x), to: coord});
                }
            }
            if Self::in_bound(y + i, x + i, 0, 3) {
                coord = Coord::new(y + i, x + i - 3);
                piece = self.field[coord.y() as usize][coord.x() as usize];
                if piece < 2 {
                    vec.push(Mov{data: 0, from: Coord::new(y, x), to: coord});
                } else if piece & 1 != color_bit {
                    vec.push(Mov{data: self.psav(piece), from: Coord::new(y, x), to: coord});
                }
            }
        }
    }

    // add all possible diagonal moves from (y, x) to vec, including captures
    fn add_legal_moves_bq(& self, vec: &mut Vec<Mov>, y: u8, x: u8, color_bit: u8) {
        let mut i: u8 = 1;
        let mut coord: Coord;
        let mut piece: u8;
        while Self::in_bound(y, x, i, i) {
            coord = Coord::new(y - i, x - i);
            piece = self.field[(y - i) as usize][(x - i) as usize];
            i += 1;
            if piece < 2 {
                vec.push(Mov{data: 0, from: Coord::new(y, x), to: coord});
            } else {
                if piece & 1 != color_bit {
                    vec.push(Mov{data: self.psav(piece), from: Coord::new(y, x), to: coord});
                }
                break;
            }
        }
        i = 1;
        while Self::in_bound(y, x + i, i, 0) {
            coord = Coord::new(y - i, x + i);
            piece = self.field[(y - i) as usize][(x + i) as usize];
            i += 1;
            if piece < 2 {
                vec.push(Mov{data: 0, from: Coord::new(y, x), to: coord});
            } else {
                if piece & 1 != color_bit {
                    vec.push(Mov{data: self.psav(piece), from: Coord::new(y, x), to: coord});
                }
                break;
            }
        }
        i = 1;
        while Self::in_bound(y + i, x + i, 0, 0) {
            coord = Coord::new(y + i, x + i);
            piece = self.field[(y + i) as usize][(x + i) as usize];
            i += 1;
            if piece < 2 {
                vec.push(Mov{data: 0, from: Coord::new(y, x), to: coord});
            } else {
                if piece & 1 != color_bit {
                    vec.push(Mov{data: self.psav(piece), from: Coord::new(y, x), to: coord});
                }
                break;
            }
        }
        i = 1;
        while Self::in_bound(y + i, x, 0, i) {
            coord = Coord::new(y + i, x - i);
            piece = self.field[(y + i) as usize][(x - i) as usize];
            i += 1;
            if piece < 2 {
                vec.push(Mov{data: 0, from: Coord::new(y, x), to: coord});
            } else {
                if piece & 1 != color_bit {
                    vec.push(Mov{data: self.psav(piece), from: Coord::new(y, x), to: coord});
                }
                break;
            }
        }
    }

    // add all possible straight moves from (y, x) to vec, including captures
    fn add_legal_moves_rq(& self, vec: &mut Vec<Mov>, y: u8, x: u8, color_bit: u8) {
        let mut i: u8 = 1;
        let mut coord: Coord;
        let mut piece: u8;
        while Self::in_bound(y, x, i, 0) {
            coord = Coord::new(y - i, x);
            piece = self.field[(y - i) as usize][x as usize];
            i += 1;
            if piece < 2 {
                vec.push(Mov{data: 0, from: Coord::new(y, x), to: coord});
            } else {
                if piece & 1 != color_bit {
                    vec.push(Mov{data: self.psav(piece), from: Coord::new(y, x), to: coord});
                }
                break;
            }
        }
        i = 1;
        while Self::in_bound(y + i, x, 0, 0) {
            coord = Coord::new(y + i, x);
            piece = self.field[(y + i) as usize][x as usize];
            i += 1;
            if piece < 2 {
                vec.push(Mov{data: 0, from: Coord::new(y, x), to: coord});
            } else {
                if piece & 1 != color_bit {
                    vec.push(Mov{data: self.psav(piece), from: Coord::new(y, x), to: coord});
                }
                break;
            }
        }
        i = 1;
        while Self::in_bound(y, x + i, 0, 0) {
            coord = Coord::new(y, x + i);
            piece = self.field[y as usize][(x + i) as usize];
            i += 1;
            if piece < 2 {
                vec.push(Mov{data: 0, from: Coord::new(y, x), to: coord});
            } else {
                if piece & 1 != color_bit {
                    vec.push(Mov{data: self.psav(piece), from: Coord::new(y, x), to: coord});
                }
                break;
            }
        }
        i = 1;
        while Self::in_bound(y, x, 0, i) {
            coord = Coord::new(y, x - i);
            piece = self.field[y as usize][(x - i) as usize];
            i += 1;
            if piece < 2 {
                vec.push(Mov{data: 0, from: Coord::new(y, x), to: coord});
            } else {
                if piece & 1 != color_bit {
                    vec.push(Mov{data: self.psav(piece), from: Coord::new(y, x), to: coord});
                }
                break;
            }
        }
    }

    // add all possible king moves from (y, x) to vec, including captures and castlings
    fn add_legal_moves_k(& self, vec: &mut Vec<Mov>, y: u8, x: u8, color_bit: u8, check_status: Option<Check>) {
        let mut coord: Coord;
        let mut piece: u8;
        for i in 0..3 {
            for j in 0..3 {
                if Self::in_bound(y + i, x + j, 1, 1) {
                    coord = Coord::new(y + i - 1, x + j - 1);
                    piece = self.field[(y + i - 1) as usize][(x + j - 1) as usize];
                    if piece < 2 {
                        vec.push(Mov{data: 0, from: Coord::new(y, x), to: coord});
                    } else if piece & 1 != color_bit {
                        vec.push(Mov{data: self.psav(piece), from: Coord::new(y, x), to: coord});
                    }
                }
            }
        }
        // this will generate not a pseudolegal move, maybe should be optimized and NOT check if king will be in check after castling
        let check: Check = check_status.unwrap_or(Check::Unknown);
        if check == Check::NotInCheck || check == Check::Unknown {
            if color_bit == 1 {
                if self.castling & self.gcl(&'K') > 0 && self.field[0][5] < 2 && self.field[0][6] < 2 {
                    if !(self.is_under_attack(0, 5, false, [true; 5]) || self.is_under_attack(0, 6, false, [true; 5])) {
                        if check == Check::NotInCheck || !self.is_under_attack(0, 4, false, [true, true, true, false, true]) {
                            vec.push(Mov{data: 1, from: Coord::new(0, 4), to: Coord::new(0, 6)});
                        }
                    }
                }
                if self.castling & self.gcl(&'Q') > 0 && self.field[0][3] < 2 && self.field[0][2] < 2 && self.field[0][1] < 2 {
                    if !(self.is_under_attack(0, 3, false, [true; 5]) || self.is_under_attack(0, 2, false, [true; 5])) {
                        if check == Check::NotInCheck || !self.is_under_attack(0, 4, false, [true, true, true, false, true]) {
                            vec.push(Mov{data: 1, from: Coord::new(0, 4), to: Coord::new(0, 2)});
                        }
                    }
                }
            } else {
                if self.castling & self.gcl(&'k') > 0 && self.field[7][5] < 2 && self.field[7][6] < 2 {
                    if !(self.is_under_attack(7, 5, true, [true; 5]) || self.is_under_attack(7, 6, true, [true; 5])) {
                        if check == Check::NotInCheck || !self.is_under_attack(7, 4, true, [true, true, true, false, true]) {
                            vec.push(Mov{data: 1, from: Coord::new(7, 4), to: Coord::new(7, 6)});
                        }
                    }
                }
                if self.castling & self.gcl(&'q') > 0 && self.field[7][3] < 2 && self.field[7][2] < 2 && self.field[7][1] < 2 {
                    if !(self.is_under_attack(7, 3, true, [true; 5]) || self.is_under_attack(7, 2, true, [true; 5])) {
                        if check == Check::NotInCheck || !self.is_under_attack(7, 4, true, [true, true, true, false, true]) {
                            vec.push(Mov{data: 1, from: Coord::new(7, 4), to: Coord::new(7, 2)});
                        }
                    }
                }
            }
        }
    }

    // add all possible pawn moves from (y, x) to vec, including captures, promotions and en passant
    fn add_legal_moves_p(& self, vec: &mut Vec<Mov>, y: u8, x: u8, color_bit: u8) {
        let mut piece: u8;
        // it's not totally different, just vertical mirrored, TODO: make it more simple!
        if color_bit == 1 {
            // promotion, promotion x capture
            if y == 6 {
                if self.field[7][x as usize] < 2 {
                    vec.push(Mov{data: self.grls(&'q') | 1, from: Coord::new(y, x), to: Coord::new(7, x)});
                    vec.push(Mov{data: self.grls(&'n') | 1, from: Coord::new(y, x), to: Coord::new(7, x)});
                    vec.push(Mov{data: self.grls(&'r') | 1, from: Coord::new(y, x), to: Coord::new(7, x)});
                    vec.push(Mov{data: self.grls(&'b') | 1, from: Coord::new(y, x), to: Coord::new(7, x)});
                }
                if Self::in_bound_single(x, 1) {
                    piece = self.field[7][(x - 1) as usize];
                    if piece > 1 && piece & 1 == 0 {
                        vec.push(Mov{data: self.grls(&'q') | self.psav(piece) | 1, from: Coord::new(y, x), to: Coord::new(7, x - 1)});
                        vec.push(Mov{data: self.grls(&'n') | self.psav(piece) | 1, from: Coord::new(y, x), to: Coord::new(7, x - 1)});
                        vec.push(Mov{data: self.grls(&'r') | self.psav(piece) | 1, from: Coord::new(y, x), to: Coord::new(7, x - 1)});
                        vec.push(Mov{data: self.grls(&'b') | self.psav(piece) | 1, from: Coord::new(y, x), to: Coord::new(7, x - 1)});
                    }
                }
                if Self::in_bound_single(x + 1, 0) {
                    piece = self.field[7][(x + 1) as usize];
                    if piece > 1 && piece & 1 == 0 {
                        vec.push(Mov{data: self.grls(&'q') | self.psav(piece) | 1, from: Coord::new(y, x), to: Coord::new(7, x + 1)});
                        vec.push(Mov{data: self.grls(&'n') | self.psav(piece) | 1, from: Coord::new(y, x), to: Coord::new(7, x + 1)});
                        vec.push(Mov{data: self.grls(&'r') | self.psav(piece) | 1, from: Coord::new(y, x), to: Coord::new(7, x + 1)});
                        vec.push(Mov{data: self.grls(&'b') | self.psav(piece) | 1, from: Coord::new(y, x), to: Coord::new(7, x + 1)});
                    }
                }
            } else {
                // 1 move forward
                // Note: this additional in_bound check might be useless (case: there is a pawn at y=8)
                if Self::in_bound_single(y + 1, 0) {
                    if self.field[(y + 1) as usize][x as usize] < 2 {
                        vec.push(Mov{data: 0, from: Coord::new(y, x), to: Coord::new(y + 1, x)});
                        // 2 moves forward
                        if y == 1 && self.field[3][x as usize] < 2 {
                            vec.push(Mov{data: 0, from: Coord::new(y, x), to: Coord::new(3, x)});
                        }
                    }
                    // simple captures
                    if Self::in_bound_single(x, 1) {
                        piece = self.field[(y + 1) as usize][(x - 1) as usize];
                        if piece > 1 && piece & 1 == 0 {
                            vec.push(Mov{data: self.psav(piece), from: Coord::new(y, x), to: Coord::new(y + 1, x - 1)});
                        }
                    }
                    if Self::in_bound_single(x + 1, 0) {
                        piece = self.field[(y + 1) as usize][(x + 1) as usize];
                        if piece > 1 && piece & 1 == 0 {
                            vec.push(Mov{data: self.psav(piece), from: Coord::new(y, x), to:  Coord::new(y + 1, x + 1)});
                        }
                    }
                }
            }
        } else {
            // basically copy-paste
            // promotion, promotion x capture
            if y == 1 {
                if self.field[0][x as usize] < 2 {
                    vec.push(Mov{data: self.grls(&'q') | 1, from: Coord::new(y, x), to: Coord::new(0, x)});
                    vec.push(Mov{data: self.grls(&'n') | 1, from: Coord::new(y, x), to: Coord::new(0, x)});
                    vec.push(Mov{data: self.grls(&'r') | 1, from: Coord::new(y, x), to: Coord::new(0, x)});
                    vec.push(Mov{data: self.grls(&'b') | 1, from: Coord::new(y, x), to: Coord::new(0, x)});
                }
                if Self::in_bound_single(x, 1) {
                    piece = self.field[0][(x - 1) as usize];
                    if piece > 1 && piece & 1 == 1 {
                        vec.push(Mov{data: self.grls(&'q') | self.psav(piece) | 1, from: Coord::new(y, x), to: Coord::new(0, x - 1)});
                        vec.push(Mov{data: self.grls(&'n') | self.psav(piece) | 1, from: Coord::new(y, x), to: Coord::new(0, x - 1)});
                        vec.push(Mov{data: self.grls(&'r') | self.psav(piece) | 1, from: Coord::new(y, x), to: Coord::new(0, x - 1)});
                        vec.push(Mov{data: self.grls(&'b') | self.psav(piece) | 1, from: Coord::new(y, x), to: Coord::new(0, x - 1)});
                    }
                }
                if Self::in_bound_single(x + 1, 0) {
                    piece = self.field[0][(x + 1) as usize];
                    if piece > 1 && piece & 1 == 1 {
                        vec.push(Mov{data: self.grls(&'q') | self.psav(piece) | 1, from: Coord::new(y, x), to: Coord::new(0, x + 1)});
                        vec.push(Mov{data: self.grls(&'n') | self.psav(piece) | 1, from: Coord::new(y, x), to: Coord::new(0, x + 1)});
                        vec.push(Mov{data: self.grls(&'r') | self.psav(piece) | 1, from: Coord::new(y, x), to: Coord::new(0, x + 1)});
                        vec.push(Mov{data: self.grls(&'b') | self.psav(piece) | 1, from: Coord::new(y, x), to: Coord::new(0, x + 1)});
                    }
                }
            } else {
                // 1 move forward
                // Note: this additional in_bound check might be useless (case: there is a pawn at y=8)
                if Self::in_bound_single(y, 1) {
                    if self.field[(y - 1) as usize][x as usize] < 2 {
                        vec.push(Mov{data: 0, from: Coord::new(y, x), to: Coord::new(y - 1, x)});
                        // 2 moves forward
                        if y == 6 && self.field[4][x as usize] < 2 {
                            vec.push(Mov{data: 0, from: Coord::new(y, x), to: Coord::new(4, x)});
                        }
                    }
                    // simple captures
                    if Self::in_bound_single(x, 1) {
                        piece = self.field[(y - 1) as usize][(x - 1) as usize];
                        if piece > 1 && piece & 1 == 1 {
                            vec.push(Mov{data: self.psav(piece), from: Coord::new(y, x), to: Coord::new(y - 1, x - 1)});
                        }
                    }
                    if Self::in_bound_single(x + 1, 0) {
                        piece = self.field[(y - 1) as usize][(x + 1) as usize];
                        if piece > 1 && piece & 1 == 1 {
                            vec.push(Mov{data: self.psav(piece), from: Coord::new(y, x), to: Coord::new(y - 1, x + 1)});
                        }
                    }
                }
            }
        }
    }

    // it's better to have it outside of add_legal_moves_p function
    fn add_legal_moves_en_passant(& self, vec: &mut Vec<Mov>) {
        if self.en_passant.y() < 8 {
            if self.en_passant.y() == 5 {
                if Board::in_bound_single(self.en_passant.x() + 1, 0) && self.field[4][self.en_passant.x() as usize + 1] == self.gpl(&'P') {
                    vec.push(Mov{ data: self.gpls(&'p') | 1, from: Coord::new(4, self.en_passant.x() + 1), to: self.en_passant });
                } else if Board::in_bound_single(self.en_passant.x(), 1) && self.field[4][self.en_passant.x() as usize - 1] == self.gpl(&'P') {
                    vec.push(Mov{ data: self.gpls(&'p') | 1, from: Coord::new(4, self.en_passant.x() - 1), to: self.en_passant });
                }
            } else if self.en_passant.y() == 2 {
                if Board::in_bound_single(self.en_passant.x() + 1, 0) && self.field[3][self.en_passant.x() as usize + 1] == self.gpl(&'p') {
                    vec.push(Mov{ data: self.gpls(&'p') | 1, from: Coord::new(3, self.en_passant.x() + 1), to: self.en_passant });
                } else if Board::in_bound_single(self.en_passant.x(), 1) && self.field[3][self.en_passant.x() as usize - 1] == self.gpl(&'p') {
                    vec.push(Mov{ data: self.gpls(&'p') | 1, from: Coord::new(3, self.en_passant.x() - 1), to: self.en_passant });
                }
            }
        }
    }

    // call this before reverting (a LEGAL move)
    fn add_check_bits(& self, mov: &mut Mov) {
        let piece = self.field[mov.to.y() as usize][mov.to.x() as usize] - !self.white_to_move as u8;
        let current_king: &Coord = self.get_current_king_coord(true);
        if piece == self.gpl(&'b') || piece == self.gpl(&'r') || piece == self.gpl(&'q') {
            if self.is_under_attack(current_king.y(), current_king.x(), !self.white_to_move, [true, true, false, false, false]) {
                mov.data |= self.bimaps.bit_check;
                // temporary impossible to trace double check in this case, see TODOs
            }
        } else if piece == self.gpl(&'k') {
            if self.is_under_attack(current_king.y(), current_king.x(), !self.white_to_move, [true, true, false, false, false]) {
                mov.data |= self.bimaps.bit_check;
            }
        } else if piece == self.gpl(&'p') {
            if self.is_under_attack(current_king.y(), current_king.x(), !self.white_to_move, [true, true, false, false, true]) {
                mov.data |= self.bimaps.bit_check;
            }
        } else if piece == self.gpl(&'n') {
            if self.is_under_attack(current_king.y(), current_king.x(), !self.white_to_move, [true, true, false, false, false]) {
                mov.data |= self.bimaps.bit_check;
            }
            if self.is_under_attack(current_king.y(), current_king.x(), !self.white_to_move, [false, false, true, false, false]) {
                if mov.data & self.bimaps.bit_check > 0 {
                    mov.data |= self.bimaps.bit_double_check;
                } else {
                    mov.data |= self.bimaps.bit_check;
                }
            }
        }
    }

    // addon methods to simplify work with the board
    // TODO: make it simple, dammit!

    // pieces by left'n'right values
    pub fn gpl(& self, piece: &char) -> u8 {
        *self.bimaps.pieces.get_by_left(piece).unwrap()
    }
    pub fn gpr(& self, value: &u8) -> char {
        *self.bimaps.pieces.get_by_right(value).unwrap()
    }
    // castles by left'n'right values
    pub fn gcl(& self, castle: &char) -> u8 {
        *self.bimaps.castles.get_by_left(castle).unwrap()
    }
    pub fn gcr(& self, value: &u8) -> char {
        *self.bimaps.castles.get_by_right(value).unwrap()
    }
    // promotions by left'n'right values
    pub fn grl(& self, promotion: &char) -> u8 {
        *self.bimaps.promotions.get_by_left(promotion).unwrap()
    }
    pub fn grr(& self, value: &u8) -> char {
        *self.bimaps.promotions.get_by_right(value).unwrap()
    }

    // get piece value by char with bit the shift to store in Mov
    pub fn gpls(& self, piece: &char) -> u8 {
        (*self.bimaps.pieces.get_by_left(piece).unwrap() & 254) << self.bimaps.shift_piece
    }
    // get promotion value by char with bit the shift to store in Mov
    pub fn grls(& self, piece: &char) -> u8 {
        *self.bimaps.promotions.get_by_left(piece).unwrap() << self.bimaps.shift_promotion
    }

    // get piece Mov value by Board value (basically transform piece to store in move)
    pub fn psav(& self, piece: u8) -> u8 {
        (piece & 254) << self.bimaps.shift_piece
    }
    // get promotion Mov value by Board value (basically transform promotion to store in move)
    pub fn rsav(& self, piece: u8) -> u8 {
        (piece & 254) << self.bimaps.shift_promotion
    }
    
    // extract piece value from move data and convert to board piece value (reminder, color's not stored)
    pub fn ptpv(& self, data: u8) -> u8 {
        (data >> self.bimaps.shift_piece) & self.bimaps.mask_piece
    }
    // extract promotion value from move data and convert to board piece value
    pub fn rtpv(& self, data: u8) -> u8 {
        self.gpl(&self.grr(&((data >> self.bimaps.shift_promotion) & self.bimaps.mask_promotion))) | self.white_to_move as u8
    }
    // extract promotion value from move data and covert to board piece char
    pub fn rtpc(& self, data: u8) -> char {
        self.grr(&((data >> self.bimaps.shift_promotion) & self.bimaps.mask_promotion))
    }

    // if this move is a promotion move (use with rptc/rtpv then) after reverting the promotion move itself)
    pub fn is_promotion(& self, mov: &Mov) -> bool {
        (mov.data & 1 > 0) && (self.field[mov.from.y() as usize][mov.from.x() as usize] == self.gpl(&'p') + self.white_to_move as u8) && (mov.to.y() == 0 || mov.to.y() == 7)
    }

    pub fn in_bound(y: u8, x: u8, y_sub: u8, x_sub: u8) -> bool {
        !(y > 7 + y_sub || x > 7 + x_sub || y_sub > y || x_sub > x)
    }
    
    pub fn in_bound_single(val: u8, sub: u8) -> bool {
        !(val > 7 + sub || sub > val)
    }
    
    fn get_default_board(bimaps: &Bimaps) -> [[u8; 8]; 8] {
        let mut field = [[0; 8]; 8];
        for i in 0..8 {
            field[1][i] = *bimaps.pieces.get_by_left(&'P').unwrap();
            field[6][i] = *bimaps.pieces.get_by_left(&'p').unwrap();
        }
        field[0][0] = *bimaps.pieces.get_by_left(&'R').unwrap();
        field[0][1] = *bimaps.pieces.get_by_left(&'N').unwrap();
        field[0][2] = *bimaps.pieces.get_by_left(&'B').unwrap();
        field[0][3] = *bimaps.pieces.get_by_left(&'Q').unwrap();
        field[0][4] = *bimaps.pieces.get_by_left(&'K').unwrap();
        field[0][5] = *bimaps.pieces.get_by_left(&'B').unwrap();
        field[0][6] = *bimaps.pieces.get_by_left(&'N').unwrap();
        field[0][7] = *bimaps.pieces.get_by_left(&'R').unwrap();
        field[7][0] = *bimaps.pieces.get_by_left(&'r').unwrap();
        field[7][1] = *bimaps.pieces.get_by_left(&'n').unwrap();
        field[7][2] = *bimaps.pieces.get_by_left(&'b').unwrap();
        field[7][3] = *bimaps.pieces.get_by_left(&'q').unwrap();
        field[7][4] = *bimaps.pieces.get_by_left(&'k').unwrap();
        field[7][5] = *bimaps.pieces.get_by_left(&'b').unwrap();
        field[7][6] = *bimaps.pieces.get_by_left(&'n').unwrap();
        field[7][7] = *bimaps.pieces.get_by_left(&'r').unwrap();
        field
    }

    // extract check status from mov data (assuming add_check_bits was called before)
    pub fn get_check(& self, data: &u8) -> Check {
        if data & self.bimaps.bit_double_check > 0 {
            return Check::InDoubleCheck;
        } else if data & self.bimaps.bit_check > 0 {
            return Check::InCheck;
        }
        Check::NotInCheck
    }

    // debug methods

    pub fn print(& self) {
        for i in 0..8 {
            for j in 0..8 {
                if self.field[7 - i][j] > 1 {
                    print!("{}\t", self.gpr(&self.field[7 - i][j]));
                } else {
                    print!(".\t");
                }
            }
            println!();
        }
        println!();
    }

    pub fn print_history(& self) {
        for bmov in &self.history {
            println!("{}", move_to_user(self, &bmov.mov));
        }
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::utils::move_to_board;

    // Tests on getting legal moves correct from a given position

    #[test]
    fn test_board_get_legal_moves_01() {
        let mut b = Board::parse_fen(&"r4nkr/1QRPPppq/2PB4/8/1n6/6N1/5PP1/1R4K1 w - - 0 1");
        let moves = b.get_legal_moves(None, None);
        assert_eq!(moves.len() == 42, true);
    }

    #[test]
    fn test_board_get_legal_moves_02() {
        let mut b = Board::parse_fen(&"r3k2r/pp1ppppp/8/8/2pP4/8/PPP1PPPP/R3K2R b KQkq d3 0 1");
        let moves = b.get_legal_moves(None, None);
        assert_eq!(moves.len() == 25, true);
    }

    #[test]
    fn test_board_get_legal_moves_03() {
        let mut b = Board::parse_fen(&"rnb1kb1r/pppppppp/4q3/8/8/3n4/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let moves = b.get_legal_moves(None, None);
        assert_eq!(moves.len() == 1, true);
    }

    #[test]
    fn test_board_get_legal_moves_04() {
        let mut b = Board::parse_fen(&"rnbqkbnr/pp1ppppp/3N4/8/8/4Q3/PPPPPPPP/RNB1KB1R b KQkq - 0 1");
        let moves = b.get_legal_moves(None, None);
        assert_eq!(moves.len() == 0, true);
    }

    #[test]
    fn test_board_get_legal_moves_05() {
        let mut b = Board::parse_fen(&"r3k2r/pp1ppppp/8/8/2pP4/3n4/PPP1PPPP/R3K2R w KQkq - 0 1");
        let moves = b.get_legal_moves(None, None);
        assert_eq!(moves.len() == 5, true);
    }

    #[test]
    fn test_board_get_legal_moves_06() {
        let mut b = Board::parse_fen(&"5k2/5ppp/5PPP/8/8/8/4R3/4R1K1 w - - 0 1");
        let moves = b.get_legal_moves(None, None);
        assert_eq!(moves.len() == 27, true);
    }

    #[test]
    fn test_board_get_legal_moves_07() {
        let mut b = Board::parse_fen(&"r3k2r/p3p2p/7n/3B4/8/8/P6P/R3K2R b KQkq - 0 1");
        let moves = b.get_legal_moves(None, None);
        assert_eq!(moves.len() == 17, true);
    }

    #[test]
    fn test_board_make_move_01() {
        let mut b = Board::new();
        b.make_move(&move_to_board(&b, &"e2e4"));
        b.make_move(&move_to_board(&b, &"b8c6"));
        b.make_move(&move_to_board(&b, &"e4e5"));
        b.make_move(&move_to_board(&b, &"d7d5"));
        b.make_move(&move_to_board(&b, &"e5d6"));
        let b2 = Board::parse_fen(&"r1bqkbnr/ppp1pppp/2nP4/8/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 3");
        for i in 0..8 {
            for j in 0..8 {
                assert_eq!(b.field[i][j] == b2.field[i][j], true);
            }
        }
        b.revert_move();
        b.revert_move();
        b.revert_move();
        b.revert_move();
        b.revert_move();
        let d = Board::get_default_board(&b.bimaps);
        for i in 0..8 {
            for j in 0..8 {
                // Yep, it's a correct test! Color bits may leave a little mess, Board will treat them as empty squares.
                assert_eq!(b.field[i][j] == d[i][j] || b.field[i][j] < 2, true);
            }
        }
    }
}