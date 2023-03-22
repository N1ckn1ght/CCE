use crate::board::bimaps::Bimaps;
use crate::board::coord::Coord;
use crate::board::mov::Mov;
use crate::board::mov::BMov;
use std::char;
use std::cmp::{max, min};
use std::mem::swap;
use std::vec::Vec;

#[derive(PartialEq, Clone, Copy)]
pub enum Check {
    Unknown,
    NotInCheck,
    InCheck,
    InDoubleCheck,
    InMate
}

#[derive(Clone)]
pub struct Board {
    // 12 - queen, 10 - rook, 8 - bishop, 6 - knight, 4 - king, 2 - pawn
    // +1 if it's a white piece
    // this choice is for better synergy with mov structure
    // white will have their pieces on 0, 1 horizontals, black on 6, 7
    pub field: [[u8; 8]; 8],
    // move storage for a takeback (revert) function
    history: Vec<BMov>,
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
    // constants to work with pieces and moves
    const B: Bimaps = Bimaps::init();

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
        let mut history: Vec<BMov> = Vec::new();
        let mut white_to_move: bool = true;
        let mut en_passant: Coord = Coord{y: 8, x : 8};
        let mut castling: u8 = 0;
        let mut hmw: u8 = 0;
        let mut no: u16 = 1;
        let mut white_king_location = Coord{y: 8, x : 8};
        let mut black_king_location = Coord{y: 8, x : 8};

        let parts = FEN.split(" ");
        let mut col: u8 = 0;
        let mut row: u8 = 7;
        let mut pn: u8 = 0;
        // since split returns lazy iterator...
        // TODO: find a better way to parse a string with spaces :/
        for part in parts {
            if pn == 0 {
                for c in part.chars() {
                    if c >= '1' && c <= '8' {
                        col += c as u8 - '0' as u8;
                    } else if c == '/' {
                        if row > 0 {
                            row -= 1;
                        }
                        col = 0;
                    } else {
                        field[row as usize][col as usize] = Board::gpl(&c);

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
                        row = c as u8 - 'a' as u8;
                        pn2 = 1;
                    } else {
                        col = c as u8 - '0' as u8;
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
    
    // Careful: this function WILL MAKE A MOVE without additional checks on if it's a legal move or not!
    // todo - promotion issue
    pub fn make_move(& self, mov: Mov) {
        let piece = self.field[mov.from.y as usize][mov.from.x as usize];

        // make a move
        self.history.push(BMov{mov, castling: self.castling, en_passant: self.en_passant, hmw: self.hmw});
        self.field[mov.to.y as usize][mov.to.x as usize] = self.field[mov.from.y as usize][mov.from.x as usize];
        self.field[mov.from.y as usize][mov.from.x as usize] = 0;

        // assuming this move is not capture or a pawn move
        self.hmw += 1;

        // update king locations + check for special cases (this one is castle)
        if piece == Board::gpl(&'k') {
            self.black_king_location = Coord{y: mov.to.y, x: mov.to.x};
            self.castling &= 192;
            if mov.data & 1 == 1 {
                if mov.to.x == 6 {
                    // Yep, it should be possible to castle even without the initial rook odd!!
                    self.field[7][5] = self.field[7][7];
                    self.field[7][7] = 0;
                } else {
                    self.field[7][3] = self.field[7][0];
                    self.field[7][0] = 0;
                }
            }
        } else if piece == Board::gpl(&'K') {
            self.white_king_location = Coord{y: mov.to.y, x: mov.to.x};
            self.castling &= 48;
            if mov.data & 1 == 1 {
                if mov.to.x == 6 {
                    self.field[0][5] = self.field[0][7];
                    self.field[0][7] = 0;
                } else {
                    self.field[0][3] = self.field[0][0];
                    self.field[0][0] = 0;
                }
            }
        } 
        // other special cases
        else if mov.data & 1 == 1 {
            // promotion or en passant
            if piece == Board::gpl(&'p') {
                // pawn's move (or capture) - drop hmw
                self.hmw = 0;
                if mov.to.y == 0 {
                    self.field[mov.to.y as usize][mov.to.x as usize] = Board::rtp(mov.data);
                } else {
                    self.field[self.en_passant.y as usize + 1][self.en_passant.x as usize] = 0;
                }
            } else { // if piece == Board::gpl(&'P')
                self.hmw = 0;
                if mov.to.y == 7 {
                    self.field[mov.to.y as usize][mov.to.x as usize] = Board::rtp(mov.data); 
                } else {
                    self.field[self.en_passant.y as usize - 1][self.en_passant.x as usize] = 0;
                }
            }
        }
        // watchout for a rook move that will prevent future castling as well
        else if piece == Board::gpl(&'r') {
            if mov.from.y == 7 {
                if mov.from.x == 0 {
                    self.castling &= 16;
                } else if mov.from.x == 7 {
                    self.castling &= 32;
                }
            }
        } else if piece == Board::gpl(&'R') {
            if mov.from.y == 0 {
                if mov.from.x == 0 {
                    self.castling &= 64;
                } else if mov.from.x == 7 {
                    self.castling &= 128;
                }
            }
        }

        // update/drop counters and next side to move
        if Board::ptp(mov.data) > 0 {
            self.hmw = 0;
        }
        self.white_to_move = !self.white_to_move;
        self.no = self.no + self.white_to_move as u16;
    }

    pub fn revert_move(& self) {
        let bmov: BMov = self.history.pop().unwrap();
        let mov: &Mov = &bmov.mov;
        let piece: u8 = self.field[mov.to.y as usize][mov.to.x as usize];
        self.field[mov.from.y as usize][mov.from.x as usize] = piece;
        self.field[mov.to.y as usize][mov.to.x as usize] = Board::ptp(mov.data);
        self.castling = bmov.castling;
        self.en_passant = bmov.en_passant;
        self.hmw = bmov.hmw;

        // reverse castling, revert kings locations
        if piece == Board::gpl(&'k') {
            self.black_king_location = Coord{y: mov.from.y, x: mov.from.x};
            if mov.to.x == 6 {
                self.field[7][7] = self.field[7][5];
                self.field[7][5] = 0;
            } else {
                self.field[7][0] = self.field[7][3];
                self.field[7][3] = 0;
            }
        } else if piece == Board::gpl(&'K') {
            self.white_king_location = Coord{y: mov.from.y, x: mov.from.x};
            if mov.data & 1 == 1 {
                if mov.to.x == 6 {
                    self.field[0][7] = self.field[0][5];
                    self.field[0][5] = 0;
                } else {
                    self.field[0][0] = self.field[0][3];
                    self.field[0][3] = 0;
                }
            }
        } else if mov.data & 1 == 1 {
            if piece - self.white_to_move as u8 == Board::gpl(&'p') {
                // now it's still other side to move, not takebacken one!
                self.field[(mov.to.y - 1 + (self.white_to_move as u8) * 2) as usize][mov.to.x as usize] = Board::gpl(&'p') + self.white_to_move as u8;
                // remove duplicated pawn?
                self.field[mov.to.y as usize][mov.to.x as usize] = 0;
            } else {
                self.field[mov.from.y as usize][mov.from.x as usize] = Board::gpl(&'p') + !self.white_to_move as u8;
            }
        }

        self.no = self.no - self.white_to_move as u16;
        self.white_to_move = !self.white_to_move;
    }

    pub fn get_legal_moves(& self, check_status: Option<Check>) -> Vec<Mov> {
        let mut moves: Vec<Mov> = vec![];
        let check: Check = check_status.unwrap_or(Check::Unknown);
        let color_bit: u8 = self.white_to_move as u8;
        match check {
            Check::Unknown | Check::InCheck => {
                // scan for any pseudo-legal moves
                for y in 0..7 {
                    for x in 0..7 {
                        if self.field[y as usize][x as usize] > 0 {
                            let piece = self.field[y as usize][x as usize] - color_bit;
                            if piece == Board::gpl(&'p') {
                                self.add_legal_moves_p(&mut moves, y, x, color_bit);
                            } else if piece == Board::gpl(&'k') {
                                // to clarify: we need only pseudo-legal moves here
                                self.add_legal_moves_k(&mut moves, y, x, color_bit, Some(Check::NotInCheck));
                            } else if piece == Board::gpl(&'n') {
                                self.add_legal_moves_n(&mut moves, y, x, color_bit);
                            } else if piece == Board::gpl(&'b') {
                                self.add_legal_moves_bq(&mut moves, y, x, color_bit);
                            } else if piece == Board::gpl(&'r') {
                                self.add_legal_moves_rq(&mut moves, y, x, color_bit);
                            } else if piece == Board::gpl(&'q') {
                                self.add_legal_moves_bq(&mut moves, y, x, color_bit);
                                self.add_legal_moves_rq(&mut moves, y, x, color_bit);
                            }
                        }
                    }
                }
                // make careful search if in check for each move for every piece!
                let mut i = 0;
                let mut len = moves.len();
                while i < len {
                    self.make_move(moves[i]);
                    let current_king: &Coord = self.get_current_king_coord(false);
                    if self.is_under_attack(current_king.y, current_king.x, self.white_to_move, [true; 5]) {
                        swap(&mut moves[i], &mut moves[len - 1]);
                        moves.pop();
                        len -= 1;
                    } else {
                        i += 1;
                    }
                }
            }
            Check::NotInCheck => {
                // still scan for any pseudo-legal moves
                for y in 0..7 {
                    for x in 0..7 {
                        if self.field[y as usize][x as usize] > 0 {
                            let piece = self.field[y as usize][x as usize] - color_bit;
                            if piece == Board::gpl(&'p') {
                                self.add_legal_moves_p(&mut moves, y, x, color_bit);
                            } else if piece == Board::gpl(&'k') {
                                self.add_legal_moves_k(&mut moves, y, x, color_bit, Some(Check::NotInCheck));
                            } else if piece == Board::gpl(&'n') {
                                self.add_legal_moves_n(&mut moves, y, x, color_bit);
                            } else if piece == Board::gpl(&'b') {
                                self.add_legal_moves_bq(&mut moves, y, x, color_bit);
                            } else if piece == Board::gpl(&'r') {
                                self.add_legal_moves_rq(&mut moves, y, x, color_bit);
                            } else if piece == Board::gpl(&'q') {
                                self.add_legal_moves_bq(&mut moves, y, x, color_bit);
                                self.add_legal_moves_rq(&mut moves, y, x, color_bit);
                            }
                        }
                    }
                }
                // make simple search on if in check
                let mut i = 0;
                let mut len = moves.len();
                while i < len {
                    self.make_move(moves[i]);
                    let current_king: &Coord = self.get_current_king_coord(false);
                    if self.is_under_attack(current_king.y, current_king.x, self.white_to_move, [true, true, true, false, false]) {
                        swap(&mut moves[i], &mut moves[len - 1]);
                        moves.pop();
                        len -= 1;
                    } else {
                        i += 1;
                    }
                }
            },
            Check::InDoubleCheck => {
                // now only king can move
                let current_king: &Coord = self.get_current_king_coord(false);
                self.add_legal_moves_k(&mut moves, current_king.y, current_king.x, self.white_to_move as u8, Some(Check::InDoubleCheck));
                // make full search on if in check
                let mut i = 0;
                let mut len = moves.len();
                while i < len {
                    self.make_move(moves[i]);
                    if self.is_under_attack(current_king.y, current_king.x, self.white_to_move, [true; 5]) {
                        swap(&mut moves[i], &mut moves[len - 1]);
                        moves.pop();
                        len -= 1;
                    } else {
                        i += 1;
                    }
                }
            },
            Check::InMate => {
                // no legal moves if it's already clear that it's a mate
                // if it's a draw, there will be no check however
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
        for i in 1..2 {
            if in_bound(y + 3, x + i, i, 0) && self.field[(y + 3 - i) as usize][(x + i)  as usize] == Board::gpl(&'b') + color_bit {
                return true;
            }
            if in_bound(y, x + 3, i, i) && self.field[(y - i) as usize][(x + 3 - i) as usize] == Board::gpl(&'n') + color_bit {
                return true;
            }
            if in_bound(y + i, x, 3, i) && self.field[(y + i - 3) as usize][(x - i) as usize] == Board::gpl(&'n') + color_bit {
                return true;
            }
            if in_bound(y + i, x + i, 0, 3) && self.field[(y + i) as usize][(x + i - 3) as usize] == Board::gpl(&'n') + color_bit {
                return true;
            }
        }
        false
    }

    // check if opponent's biship or queen is attacking this cell by diagonal
    fn is_under_attack_bq(& self, y: u8, x: u8, color_bit: u8) -> bool {
        let mut i: u8 = 1;
        let mut piece: u8;
        while in_bound(y, x, i, i) {
            piece = self.field[(y - i) as usize][(x - i) as usize];
            i += 1;
            if piece > 0 {
                piece -= color_bit;
            } else {
                continue;
            }
            if piece == Board::gpl(&'b') || piece == Board::gpl(&'q') {
                return true;
            } else {
                break;
            }
        }
        i = 1;
        while in_bound(y, x + i, i, 0) {
            piece = self.field[(y - i) as usize][(x + i) as usize];
            i += 1;
            if piece > 0 {
                piece -= color_bit;
            } else {
                continue;
            }
            if piece == Board::gpl(&'b') || piece == Board::gpl(&'q') {
                return true;
            } else {
                break;
            }
        }
        i = 1;
        while in_bound(y + i, x + i, 0, 0) {
            piece = self.field[(y + i) as usize][(x + i) as usize];
            i += 1;
            if piece > 0 {
                piece -= color_bit;
            } else {
                continue;
            }
            if piece == Board::gpl(&'b') || piece == Board::gpl(&'q') {
                return true;
            } else {
                break;
            }
        }
        i = 1;
        while in_bound(y + i, x, 0, i) {
            piece = self.field[(y + i) as usize][(x - i) as usize];
            i += 1;
            if piece > 0 {
                piece -= color_bit;
            } else {
                continue;
            }
            if piece == Board::gpl(&'b') || piece == Board::gpl(&'q') {
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
        while in_bound(y, x, i, 0) {
            piece = self.field[(y - i) as usize][x as usize];
            i += 1;
            if piece > 0 {
                piece -= color_bit;
            } else {
                continue;
            }
            if piece == Board::gpl(&'r') || piece == Board::gpl(&'q') {
                return true;
            } else {
                break;
            }
        }
        i = 1;
        while in_bound(y + i, x, 0, 0) {
            piece = self.field[(y + i) as usize][x as usize];
            i += 1;
            if piece > 0 {
                piece -= color_bit;
            } else {
                continue;
            }
            if piece == Board::gpl(&'r') || piece == Board::gpl(&'q') {
                return true;
            } else {
                break;
            }
        }
        i = 1;
        while in_bound(y, x + i, 0, 0) {
            piece = self.field[y as usize][(x + i) as usize];
            i += 1;
            if piece > 0 {
                piece -= color_bit;
            } else {
                continue;
            }
            if piece == Board::gpl(&'r') || piece == Board::gpl(&'q') {
                return true;
            } else {
                break;
            }
        }
        i = 1;
        while in_bound(y, x, 0, i) {
            piece = self.field[y as usize][(x - i) as usize];
            i += 1;
            if piece > 0 {
                piece -= color_bit;
            } else {
                continue;
            }
            if piece == Board::gpl(&'r') || piece == Board::gpl(&'q') {
                return true;
            } else {
                break;
            }
        }
        false
    }

    // check if opponent's king is attacking this cell
    fn is_under_attack_k(& self, y: u8, x: u8, color_of_attacker: bool) -> bool {
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
    fn is_under_attack_p(& self, y: u8, x: u8, color_of_attacker: bool) -> bool {
        if color_of_attacker {
            if in_bound(y + 1, x + 1, 0, 0) && self.field[(y + 1) as usize][(x + 1) as usize] == Board::gpl(&'P') {
                return true;
            }
            if in_bound(y + 1, x, 0, 1) && self.field[(y + 1) as usize][(x - 1) as usize] == Board::gpl(&'P') {
                return true;
            }
        } else {
            if in_bound(y, x + 1, 1, 0) && self.field[(y - 1) as usize][(x + 1) as usize] == Board::gpl(&'p') {
                return true;
            }
            if in_bound(y, x, 1, 1) && self.field[(y - 1) as usize][(x - 1) as usize] == Board::gpl(&'p') {
                return true;
            }
        }
        // check for en_passant, though this is a very specific case that may never be used
        self.en_passant.y == y && self.en_passant.x == x
    }

    // if possible square for a knight is empty or has a piece of not a color_bit color, this move will be added
    fn add_legal_moves_n(& self, vec: &mut Vec<Mov>, y: u8, x: u8, color_bit: u8) {
        let mut coord: Coord;
        let mut piece: u8;
        for i in 1..2 {
            if in_bound(y + 3, x + i, i, 0) {
                coord = Coord{y: y + 3 - i, x: x + i};
                piece = self.field[coord.y as usize][coord.x as usize];
                if piece == 0 {
                    vec.push(Mov{data: 0, from: Coord{y, x}, to: coord});
                } else if piece & 1 != color_bit {
                    vec.push(Mov{data: Board::sav(piece) , from: Coord{y, x}, to: coord});
                }
            }
            if in_bound(y, x + 3, i, i) {
                coord = Coord{y: y - i, x: x + 3 - i};
                piece = self.field[coord.y as usize][coord.x as usize];
                if piece == 0 {
                    vec.push(Mov{data: 0, from: Coord{y, x}, to: coord});
                } else if piece & 1 != color_bit {
                    vec.push(Mov{data: Board::sav(piece), from: Coord{y, x}, to: coord});
                }
            }
            if in_bound(y + i, x, 3, i) {
                coord = Coord{y: y + i - 3, x: x - i};
                piece = self.field[coord.y as usize][coord.x as usize];
                if piece == 0 {
                    vec.push(Mov{data: 0, from: Coord{y, x}, to: coord});
                } else if piece & 1 != color_bit {
                    vec.push(Mov{data: Board::sav(piece), from: Coord{y, x}, to: coord});
                }
            }
            if in_bound(y + i, x + i, 0, 3) {
                coord = Coord{y: y + i, x: x + i - 3};
                piece = self.field[coord.y as usize][coord.x as usize];
                if piece == 0 {
                    vec.push(Mov{data: 0, from: Coord{y, x}, to: coord});
                } else if piece & 1 != color_bit {
                    vec.push(Mov{data: Board::sav(piece), from: Coord{y, x}, to: coord});
                }
            }
        }
    }

    // add all possible diagonal moves from (y, x) to vec, including captures
    fn add_legal_moves_bq(& self, vec: &mut Vec<Mov>, y: u8, x: u8, color_bit: u8) {
        let mut i: u8 = 1;
        let mut coord: Coord;
        let mut piece: u8;
        while in_bound(y, x, i, i) {
            coord = Coord{y: y - i, x: x - i};
            piece = self.field[(y - i) as usize][(x - i) as usize];
            i += 1;
            if piece == 0 {
                vec.push(Mov{data: 0, from: Coord{y, x}, to: coord});
            } else if piece & 1 != color_bit {
                vec.push(Mov{data: Board::sav(piece), from: Coord{y, x}, to: coord});
            } else {
                break;
            }
        }
        i = 1;
        while in_bound(y, x + i, i, 0) {
            coord = Coord{y: y - i, x: x + i};
            piece = self.field[(y - i) as usize][(x + i) as usize];
            i += 1;
            if piece == 0 {
                vec.push(Mov{data: 0, from: Coord{y, x}, to: coord});
            } else if piece & 1 != color_bit {
                vec.push(Mov{data: Board::sav(piece), from: Coord{y, x}, to: coord});
            } else {
                break;
            }
        }
        i = 1;
        while in_bound(y + i, x + i, 0, 0) {
            coord = Coord{y: y + i, x: x + i};
            piece = self.field[(y + i) as usize][(x + i) as usize];
            i += 1;
            if piece == 0 {
                vec.push(Mov{data: 0, from: Coord{y, x}, to: coord});
            } else if piece & 1 != color_bit {
                vec.push(Mov{data: Board::sav(piece), from: Coord{y, x}, to: coord});
            } else {
                break;
            }
        }
        i = 1;
        while in_bound(y + i, x, 0, i) {
            coord = Coord{y: y + i, x: x - i};
            piece = self.field[(y + i) as usize][(x - i) as usize];
            i += 1;
            if piece == 0 {
                vec.push(Mov{data: 0, from: Coord{y, x}, to: coord});
            } else if piece & 1 != color_bit {
                vec.push(Mov{data: Board::sav(piece), from: Coord{y, x}, to: coord});
            } else {
                break;
            }
        }
    }

    // add all possible straight moves from (y, x) to vec, including captures
    fn add_legal_moves_rq(& self, vec: &mut Vec<Mov>, y: u8, x: u8, color_bit: u8) {
        let mut i: u8 = 1;
        let mut coord: Coord;
        let mut piece: u8;
        while in_bound(y, x, i, 0) {
            coord = Coord{y: y - i, x: x};
            piece = self.field[(y - i) as usize][x as usize];
            i += 1;
            if piece == 0 {
                vec.push(Mov{data: 0, from: Coord{y, x}, to: coord});
            } else if piece & 1 != color_bit {
                vec.push(Mov{data: Board::sav(piece), from: Coord{y, x}, to: coord});
            } else {
                break;
            }
        }
        i = 1;
        while in_bound(y + i, x, 0, 0) {
            coord = Coord{y: y + i, x: x};
            piece = self.field[(y + i) as usize][x as usize];
            i += 1;
            if piece == 0 {
                vec.push(Mov{data: 0, from: Coord{y, x}, to: coord});
            } else if piece & 1 != color_bit {
                vec.push(Mov{data: Board::sav(piece), from: Coord{y, x}, to: coord});
            } else {
                break;
            }
        }
        i = 1;
        while in_bound(y, x + i, 0, 0) {
            coord = Coord{y: y, x: x + i};
            piece = self.field[y as usize][(x + i) as usize];
            i += 1;
            if piece == 0 {
                vec.push(Mov{data: 0, from: Coord{y, x}, to: coord});
            } else if piece & 1 != color_bit {
                vec.push(Mov{data: Board::sav(piece), from: Coord{y, x}, to: coord});
            } else {
                break;
            }
        }
        i = 1;
        while in_bound(y, x, 0, i) {
            coord = Coord{y: y, x: x - i};
            piece = self.field[y as usize][(x - i) as usize];
            i += 1;
            if piece == 0 {
                vec.push(Mov{data: 0, from: Coord{y, x}, to: coord});
            } else if piece & 1 != color_bit {
                vec.push(Mov{data: Board::sav(piece), from: Coord{y, x}, to: coord});
            } else {
                break;
            }
        }
    }

    // add all possible king moves from (y, x) to vec, including captures and castlings
    fn add_legal_moves_k(& self, vec: &mut Vec<Mov>, y: u8, x: u8, color_bit: u8, check_status: Option<Check>) {
        let mut coord: Coord;
        let mut piece: u8;
        for i in 0..2 {
            for j in 0..2 {
                if in_bound(y + i, x + j, 1, 1) {
                    coord = Coord{y: y + i - 1, x: x + j - 1};
                    piece = self.field[(y + i - 1) as usize][(x + j - 1) as usize];
                    if piece == 0 {
                        vec.push(Mov{data: 0, from: Coord{y, x}, to: coord});
                    } else if piece & 1 != color_bit {
                        vec.push(Mov{data: Board::sav(piece), from: Coord{y, x}, to: coord});
                    }
                }
            }
        }
        // this will generate not a pseudolegal move, maybe should be optimized and NOT check if king will be in check after castling
        let check: Check = check_status.unwrap_or(Check::Unknown);
        if check == Check::NotInCheck || check == Check::Unknown {
            if color_bit == 1 {
                if self.castling & Board::gcl(&'K') > 0 && self.field[0][5] == 0 && self.field[0][6] == 0 {
                    if !(self.is_under_attack(0, 5, false, [true; 5]) || self.is_under_attack(0, 6, false, [true; 5])) {
                        if check == Check::NotInCheck || !self.is_under_attack(0, 4, false, [true, true, true, false, true]) {
                            vec.push(Mov{data: 1, from: Coord{y: 0, x: 4}, to: Coord{y: 0, x: 6}});
                        }
                    }
                }
                if self.castling & Board::gcl(&'Q') > 0 && self.field[0][3] == 0 && self.field[0][2] == 0 && self.field[0][1] == 0 {
                    if !(self.is_under_attack(0, 3, false, [true; 5]) || self.is_under_attack(0, 2, false, [true; 5])) {
                        if check == Check::NotInCheck || !self.is_under_attack(0, 4, false, [true, true, true, false, true]) {
                            vec.push(Mov{data: 1, from: Coord{y: 0, x: 4}, to: Coord{y: 0, x: 2}});
                        }
                    }
                }
            } else {
                if self.castling & Board::gcl(&'k') > 0 && self.field[7][5] == 0 && self.field[7][6] == 0 {
                    if check == Check::NotInCheck || !self.is_under_attack(7, 4, false, [true, true, true, false, true]) {
                        vec.push(Mov{data: 1, from: Coord{y: 7, x: 4}, to: Coord{y: 7, x: 6}});
                    }
                }
                if self.castling & Board::gcl(&'q') > 0 && self.field[7][3] == 0 && self.field[7][2] == 0 && self.field[7][1] == 0 {
                    if check == Check::NotInCheck || !self.is_under_attack(7, 4, false, [true, true, true, false, true]) {
                        vec.push(Mov{data: 1, from: Coord{y: 7, x: 4}, to: Coord{y: 7, x: 2}});
                    }
                }
            }
        }
    }

    // add all possible pawn moves from (y, x) to vec, including captures, promotions and en passant
    fn add_legal_moves_p(& self, vec: &mut Vec<Mov>, y: u8, x: u8, color_bit: u8) {
        let mut piece: u8;
        if color_bit == 1 {
            // promotion, promotion x capture
            if y == 6 {
                if self.field[7][x as usize] == 0 {
                    vec.push(Mov{data: Board::grl(&'q'), from: Coord{y, x}, to: Coord{y: 7, x}});
                    vec.push(Mov{data: Board::grl(&'n'), from: Coord{y, x}, to: Coord{y: 7, x}});
                    vec.push(Mov{data: Board::grl(&'r'), from: Coord{y, x}, to: Coord{y: 7, x}});
                    vec.push(Mov{data: Board::grl(&'b'), from: Coord{y, x}, to: Coord{y: 7, x}});
                }
                if in_bound_single(x, 1) {
                    piece = self.field[7][(x - 1) as usize];
                    if piece > 0 && piece & 1 == 0 {
                        vec.push(Mov{data: Board::grl(&'q') + Board::sav(piece), from: Coord{y, x}, to: Coord{y: 7, x: x - 1}});
                        vec.push(Mov{data: Board::grl(&'n') + Board::sav(piece), from: Coord{y, x}, to: Coord{y: 7, x: x - 1}});
                        vec.push(Mov{data: Board::grl(&'r') + Board::sav(piece), from: Coord{y, x}, to: Coord{y: 7, x: x - 1}});
                        vec.push(Mov{data: Board::grl(&'b') + Board::sav(piece), from: Coord{y, x}, to: Coord{y: 7, x: x - 1}});
                    }
                }
                if in_bound_single(x + 1, 0) {
                    piece = self.field[7][(x + 1) as usize];
                    if piece > 0 && piece & 1 == 0 {
                        vec.push(Mov{data: Board::grl(&'q') + Board::sav(piece), from: Coord{y, x}, to: Coord{y: 7, x: x + 1}});
                        vec.push(Mov{data: Board::grl(&'n') + Board::sav(piece), from: Coord{y, x}, to: Coord{y: 7, x: x + 1}});
                        vec.push(Mov{data: Board::grl(&'r') + Board::sav(piece), from: Coord{y, x}, to: Coord{y: 7, x: x + 1}});
                        vec.push(Mov{data: Board::grl(&'b') + Board::sav(piece), from: Coord{y, x}, to: Coord{y: 7, x: x + 1}});
                    }
                }
            }
            // 1 move forward
            // Note: this additional in_bound check might be useless (case: there is a pawn at y=8)
            if in_bound_single(y + 1, 0) {
                if self.field[(y + 1) as usize][x as usize] == 0 {
                    vec.push(Mov{data: 0, from: Coord {y, x}, to: Coord{y: y + 1, x}});
                    // 2 moves forward
                    if y == 1 && self.field[3][x as usize] == 0 {
                        vec.push(Mov{data: 0, from: Coord {y, x}, to: Coord{y: 3, x}});
                    }
                }
                // simple captures
                if in_bound_single(x, 1) {
                    piece = self.field[(y + 1) as usize][(x - 1) as usize];
                    if piece > 0 && piece & 1 == 0 {
                        vec.push(Mov{data: Board::sav(piece), from: Coord{y, x}, to: Coord{y: y + 1, x: x - 1}});
                    }
                }
                if in_bound_single(x + 1, 0) {
                    piece = self.field[(y + 1) as usize][(x + 1) as usize];
                    if piece > 0 && piece & 1 == 0 {
                        vec.push(Mov{data: Board::sav(piece), from: Coord{y, x}, to: Coord{y: y + 1, x: x + 1}});
                    }
                }
            }
            // en passant
            if self.en_passant.y == 5 && y == 4 {
                if x + 1 == self.en_passant.x || x == self.en_passant.x + 1 {
                    vec.push(Mov{data: Board::gpls(&'p') + 1, from: Coord{y, x}, to: self.en_passant.clone()});
                }
            }
        } else {
            // basically copy-paste
            // promotion, promotion x capture
            if y == 1 {
                if self.field[0][x as usize] == 0 {
                    vec.push(Mov{data: Board::grl(&'q'), from: Coord{y, x}, to: Coord{y: 0, x}});
                    vec.push(Mov{data: Board::grl(&'n'), from: Coord{y, x}, to: Coord{y: 0, x}});
                    vec.push(Mov{data: Board::grl(&'r'), from: Coord{y, x}, to: Coord{y: 0, x}});
                    vec.push(Mov{data: Board::grl(&'b'), from: Coord{y, x}, to: Coord{y: 0, x}});
                }
                if in_bound_single(x, 1) {
                    piece = self.field[0][(x - 1) as usize];
                    if piece > 0 && piece & 1 == 1 {
                        vec.push(Mov{data: Board::grl(&'q') + Board::sav(piece), from: Coord{y, x}, to: Coord{y: 0, x: x - 1}});
                        vec.push(Mov{data: Board::grl(&'n') + Board::sav(piece), from: Coord{y, x}, to: Coord{y: 0, x: x - 1}});
                        vec.push(Mov{data: Board::grl(&'r') + Board::sav(piece), from: Coord{y, x}, to: Coord{y: 0, x: x - 1}});
                        vec.push(Mov{data: Board::grl(&'b') + Board::sav(piece), from: Coord{y, x}, to: Coord{y: 0, x: x - 1}});
                    }
                }
                if in_bound_single(x + 1, 0) {
                    piece = self.field[0][(x + 1) as usize];
                    if piece > 0 && piece & 1 == 1 {
                        vec.push(Mov{data: Board::grl(&'q') + Board::sav(piece), from: Coord{y, x}, to: Coord{y: 0, x: x + 1}});
                        vec.push(Mov{data: Board::grl(&'n') + Board::sav(piece), from: Coord{y, x}, to: Coord{y: 0, x: x + 1}});
                        vec.push(Mov{data: Board::grl(&'r') + Board::sav(piece), from: Coord{y, x}, to: Coord{y: 0, x: x + 1}});
                        vec.push(Mov{data: Board::grl(&'b') + Board::sav(piece), from: Coord{y, x}, to: Coord{y: 0, x: x + 1}});
                    }
                }
            }
            // 1 move forward
            // Note: this additional in_bound check might be useless (case: there is a pawn at y=8)
            if in_bound_single(y, 1) {
                if self.field[(y - 1) as usize][x as usize] == 0 {
                    vec.push(Mov{data: 0, from: Coord {y, x}, to: Coord{y: y - 1, x}});
                    // 2 moves forward
                    if y == 6 && self.field[4][x as usize] == 0 {
                        vec.push(Mov{data: 0, from: Coord {y, x}, to: Coord{y: 4, x}});
                    }
                }
                // simple captures
                if in_bound_single(x, 1) {
                    piece = self.field[(y - 1) as usize][(x - 1) as usize];
                    if piece > 0 && piece & 1 == 1 {
                        vec.push(Mov{data: Board::sav(piece), from: Coord{y, x}, to: Coord{y: y - 1, x: x - 1}});
                    }
                }
                if in_bound_single(x + 1, 0) {
                    piece = self.field[(y - 1) as usize][(x + 1) as usize];
                    if piece > 0 && piece & 1 == 1 {
                        vec.push(Mov{data: Board::sav(piece), from: Coord{y, x}, to: Coord{y: y - 1, x: x + 1}});
                    }
                }
            }
            // en passant
            if self.en_passant.y == 2 && y == 3 {
                if x + 1 == self.en_passant.x || x == self.en_passant.x + 1 {
                    vec.push(Mov{data: Board::gpls(&'p') + 1, from: Coord{y, x}, to: self.en_passant.clone()});
                }
            }
        }
    }

    // functions to simplify work with bimaps

    // pieces by left'n'right values
    pub fn gpl(piece: &char) -> u8 {
        Board::B.pieces.get_by_left(piece).unwrap().clone()
    }
    pub fn gpr(value: &u8) -> char {
        Board::B.pieces.get_by_right(value).unwrap().clone()
    }
    // castles by left'n'right values
    pub fn gcl(castle: &char) -> u8 {
        Board::B.castles.get_by_left(castle).unwrap().clone()
    }
    pub fn gcr(value: &u8) -> char {
        Board::B.castles.get_by_right(value).unwrap().clone()
    }
    // promotions by left'n'right values
    pub fn grl(promotion: &char) -> u8 {
        Board::B.promotions.get_by_left(promotion).unwrap().clone()
    }
    pub fn grr(value: &u8) -> char {
        Board::B.promotions.get_by_right(value).unwrap().clone()
    }

    // and more

    // get piece value by char with bit the shift to store in Mov
    pub fn gpls(piece: &char) -> u8 {
        (Board::B.pieces.get_by_left(piece).unwrap().clone() & 254) << Board::B.shift_piece
    }
    // get piece Mov value by Board value (basically transform piece to store in move)
    pub fn sav(piece: u8) -> u8 {
        (piece & 254) << Board::B.shift_piece
    }
    // extract piece value from move data and convert to board piece value (reminder, color's not stored)
    pub fn ptp(data: u8) -> u8 {
        (data >> Board::B.shift_piece) & Board::B.mask_piece
    }
    // extract promotion value from move data and convert to board piece value
    pub fn rtp(data: u8) -> u8 {
        Board::gpl(&Board::grr(&((data >> Board::B.shift_promotion) & Board::B.mask_promotion)))
    }
}

pub fn in_bound(y: u8, x: u8, y_sub: u8, x_sub: u8) -> bool {
    !(y > 7 + y_sub || x > 7 + x_sub || y_sub > y || x_sub > x)
}

pub fn in_bound_single(val: u8, sub: u8) -> bool {
    !(val > 7 + sub || sub > val)
}

fn get_default_board() -> [[u8; 8]; 8] {
    let mut field = [[0; 8]; 8];
    for i in 0..7 {
        field[1][i] = Board::gpl(&'P');
        field[6][i] = Board::gpl(&'p');
    }
    field[0][0] = Board::gpl(&'R');
    field[0][1] = Board::gpl(&'N');
    field[0][2] = Board::gpl(&'B');
    field[0][3] = Board::gpl(&'Q');
    field[0][4] = Board::gpl(&'K');
    field[0][5] = Board::gpl(&'B');
    field[0][6] = Board::gpl(&'N');
    field[0][7] = Board::gpl(&'R');
    field[7][0] = Board::gpl(&'r');
    field[7][1] = Board::gpl(&'n');
    field[7][2] = Board::gpl(&'b');
    field[7][3] = Board::gpl(&'q');
    field[7][4] = Board::gpl(&'k');
    field[7][5] = Board::gpl(&'b');
    field[7][6] = Board::gpl(&'n');
    field[7][7] = Board::gpl(&'r');
    field
}