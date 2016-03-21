use position::Position;
use piece_move::Move;

#[derive(Clone)]
struct PlayerState {
    can_castle_short: bool,
    can_castle_long: bool,
}

fn initial_player_state() -> PlayerState {
    PlayerState {
        can_castle_short: true,
        can_castle_long: true,
    }
}

#[derive(Clone)]
pub struct GameState {
    board: [[Option<Piece>; 8]; 8],
    current_player: Color,
    white_state: PlayerState,
    black_state: PlayerState,
    en_passant_target: Option<Position>,
}

impl GameState {
    pub fn opening_state() -> GameState {
        // White on top so that (0,0) matches up with a1. Flipped for the actual display.
        let raw_board =
            [['R', 'N', 'B', 'Q', 'K', 'B', 'N', 'R']
            ,['P', 'P', 'P', 'P', 'P', 'P', 'P', 'P']
            ,['-', '-', '-', '-', '-', '-', '-', '-']
            ,['-', '-', '-', '-', '-', '-', '-', '-']
            ,['-', '-', '-', '-', '-', '-', '-', '-']
            ,['-', '-', '-', '-', '-', '-', '-', '-']
            ,['p', 'p', 'p', 'p', 'p', 'p', 'p', 'p']
            ,['r', 'n', 'b', 'q', 'k', 'b', 'n', 'r']];

        let mut board = [[Option::None; 8]; 8];
        for row in 0..8 {
            for column in 0..8 {
                board[row][column] = raw_board[row][column].to_piece();
            }
        }

        GameState {
            board: board,
            current_player: Color::White,
            white_state: initial_player_state(),
            black_state: initial_player_state(),
            en_passant_target: Option::None,
        }
    }

    pub fn format(&self) -> String {
        let mut result: String = String::new();
        for row in (0..8).rev() {
            for column in 0..8 {
                result.push(self.board[row][column].map_or('-', |piece| piece.to_char()));
                result.push(' ');
            }

            result.push('\n');
        }

        result
    }

    pub fn play_turn(&mut self, player_brain: Box<Fn(&GameState) -> Move>) {
        let player_move = player_brain(&self.clone());
        self.move_piece(&player_move);

        self.current_player = if self.current_player == Color::White {
            Color::Black
        } else {
            Color::White
        };
    }

    fn get_piece(&self, position: &Position) -> Option<Piece> {
        self.board[position.row as usize][position.column as usize]
    }

    fn set_piece(&mut self, piece: &Option<Piece>, position: &Position) {
        self.board[position.row as usize][position.column as usize] = *piece;
    }

    fn move_piece(&mut self, player_move: &Move) {
        // En passant is only possible for the turn after it was enabled.
        self.en_passant_target = None;

        let source_piece = self.get_piece(&player_move.source);
        self.set_piece(&source_piece, &player_move.destination);
        self.set_piece(&None, &player_move.source);

        if let Some(en_passant_target) = player_move.en_passant_target.clone() {
            self.set_piece(&None, &en_passant_target);
        }

        if player_move.enables_en_passant {
            self.en_passant_target = Some(player_move.destination.clone());
        }
    }

    fn is_in_bounds(&self, position: &Position) -> bool {
        position.column >= 0 && position.row >= 0 && position.column < 8 && position.row < 8
    }

    fn get_occupation_status(&self, friendly_piece: &Piece, dest: &Position) -> OccupationStatus {
        match self.get_piece(dest) {
            None => OccupationStatus::Empty,
            Some(piece) if piece.color == friendly_piece.color => OccupationStatus::Friendly,
            Some(_) => OccupationStatus::Enemy,
        }
    }

    pub fn get_current_player_moves(&self) -> Vec<Move> {
        let mut moves = vec![];
        for row in 0..8 {
            for column in 0..8 {
                let position = &Position { column: column, row: row };
                let piece = self.get_piece(position);
                if piece.map_or(false, |p| p.color == self.current_player) {
                    moves.append(&mut self.get_moves_for_piece(position));
                }
            }
        }

        moves
    }

    fn get_moves_for_piece(&self, source: &Position) -> Vec<Move> {
        let maybe_piece = self.get_piece(source);
        if maybe_piece.is_none() {
            return vec![];
        }

        let piece = maybe_piece.unwrap();
        let mut dests = vec![];
        
        match piece.piece_type {
            PieceType::Pawn => {
                let mut moves = vec![];

                let direction = if piece.color == Color::White { 1 } else { -1 };
                let start_row = if piece.color == Color::White { 1 } else {  6 };

                if let Some(forward_one) = self.relative(&source, 0, direction) {
                    if self.get_occupation_status(&piece, &forward_one) == OccupationStatus::Empty {
                        moves.push(Move::simple(source.clone(), forward_one));
                        if let Some(forward_two) = self.relative(source, 0, 2 * direction) {
                            // Pawns can move forward only one unless they are in their starting row.
                            if source.row == start_row && self.get_occupation_status(&piece, &forward_two)
                                    == OccupationStatus::Empty {
                                moves.push(Move::en_passant_enabler(source.clone(), forward_two));
                            }
                        }
                    }
                }

                // Pawns can take pieces on diagonals immediately in front of them.
                if let Some(left_attack) = self.relative(&source, -1, direction) {
                    if self.get_occupation_status(&piece, &left_attack) == OccupationStatus::Enemy {
                        moves.push(Move::simple(source.clone(), left_attack));
                    }
                }

                if let Some(right_attack) = self.relative(&source, 1, direction) {
                    if self.get_occupation_status(&piece, &right_attack) == OccupationStatus::Enemy {
                        moves.push(Move::simple(source.clone(), right_attack));
                    }
                }

                if let Some(left_en_passant) = self.relative(&source, -1, 0) {
                    if self.en_passant_target == Some(left_en_passant.clone()) {
                        moves.push(Move::en_passant(
                                source.clone(), source.relative(-1, direction), left_en_passant));
                    }
                }
                
                if let Some(right_en_passant) = self.relative(&source, 1, 0) {
                    if self.en_passant_target == Some(right_en_passant.clone()) {
                        moves.push(Move::en_passant(
                                source.clone(), source.relative(1, direction), right_en_passant));
                    }
                }

                return moves;
            },

            PieceType::Knight => {
                dests.append(&mut self.get_consecutive_dests(
                        source,
                        &[(-2, -1), (-2, 1), (-1, -2), (-1, 2), (1, -2), (1, 2), (2, -1), (2, 1)],
                        Some(1) 
                        ));
            },

            PieceType::Bishop => {
                dests.append(&mut self.get_consecutive_dests(
                        source,
                        &[(-1, -1), (-1, 1), (1, -1), (1, 1)],
                        None
                        ));
            },

            PieceType::Rook => {
                dests.append(&mut self.get_consecutive_dests(
                        source,
                        &[(-1, 0), (0, -1), (0, 1), (1, 0)],
                        None
                        ));
            },

            PieceType::Queen => {
                dests.append(&mut self.get_consecutive_dests(
                        source,
                        &[(-1,-1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)],
                        None
                        ));

            },
            PieceType::King => {
                dests.append(&mut self.get_consecutive_dests(
                        source,
                        &[(-1,-1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)],
                        Some(1)
                        ));

            },
        }

        dests.iter().map(|dest| Move::simple(source.clone(), dest.clone())).collect::<Vec<_>>()
    }

    fn get_consecutive_dests(
            &self,
            source: &Position,
            dirs: &[(i8, i8)],
            max_moves: Option<u8>) -> Vec<Position> {

        let source_piece = &self.get_piece(source).unwrap();

        let mut dests = vec![];

        for &(col_dir, row_dir) in dirs {
            if col_dir == 0 && row_dir == 0 {
                panic!("at least one dir must be non-zero");
            }

            let mut current_dir_dests = vec![];
            let mut dest = source.clone();
            loop {
                dest = dest.relative(col_dir, row_dir);
                if !self.is_in_bounds(&dest)
                        || max_moves.map_or(false, |max| current_dir_dests.len() >= max as usize) {
                    break;
                }
                

                match self.get_occupation_status(source_piece, &dest) {
                    OccupationStatus::Friendly => break,
                    OccupationStatus::Empty => dests.push(dest.clone()),
                    OccupationStatus::Enemy => {
                        current_dir_dests.push(dest.clone());
                        break;
                    },
                }
            }

            dests.append(&mut current_dir_dests);
        }

        dests
    }

    fn relative(&self, source: &Position, col_offset: i8, row_offset: i8) -> Option<Position> {
        let result = source.relative(col_offset, row_offset);
        if self.is_in_bounds(&result) {
            Some(result)
        } else {
            None
        }
    }
}

#[derive(PartialEq)]
enum OccupationStatus {
    Empty,
    Friendly,
    Enemy,
}

#[derive(Copy, Clone)]
struct Piece {
    color: Color,
    piece_type: PieceType,
}

impl Piece {
    fn to_char(&self) -> char {
        let result = match self.piece_type {
            PieceType::Pawn   => 'p',
            PieceType::Knight => 'n',
            PieceType::Bishop => 'b',
            PieceType::Rook   => 'r',
            PieceType::Queen  => 'q',
            PieceType::King   => 'k',
        };

        if self.color == Color::White { 
            result.to_uppercase().next().unwrap()
        } else {
            result
        }
    }
}

trait ToPiece {
    fn to_piece(&self) -> Option<Piece>;
}

impl ToPiece for char {
    fn to_piece(&self) -> Option<Piece> {
        if *self == '-' {
            None
        } else {
            Some(Piece {
                color: if self.is_uppercase() { Color::White } else { Color::Black },
                piece_type: match self.to_uppercase().next().unwrap() {
                    'P' => PieceType::Pawn,
                    'N' => PieceType::Knight,
                    'B' => PieceType::Bishop,
                    'R' => PieceType::Rook,
                    'Q' => PieceType::Queen,
                    'K' => PieceType::King,
                    _   => unreachable!()
                }
            })
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King
}

#[derive(PartialEq, Clone, Copy)]
enum Color {
    White,
    Black
}
