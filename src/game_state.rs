use piece_type::PieceType;
use position::Position;
use piece_move::Move;

static NONE: Option<Move<'static>> = None;

static WHITE_LEFT_ROOK_MOVE: Option<Move<'static>> =
    Some(Move {
        source: Position{column: 0, row: 0},
        destination: Position {column: 3, row: 0},
        extra_castling_move: &NONE,
        enables_en_passant: false,
        en_passant_target: None,
        promotion_piece_type: None,
        });
static WHITE_LEFT_CASTLE: Move<'static> =
    Move {
        source: Position {column: 4, row: 0},
        destination: Position {column: 2, row: 0},
        extra_castling_move: &WHITE_LEFT_ROOK_MOVE,
        enables_en_passant: false,
        en_passant_target: None,
        promotion_piece_type: None, 
    };

static WHITE_RIGHT_ROOK_MOVE: Option<Move<'static>> =
    Some(Move {
        source: Position {column: 7, row: 0},
        destination: Position {column: 5, row: 0},
        extra_castling_move: &NONE,
        enables_en_passant: false,
        en_passant_target: None,
        promotion_piece_type: None, 
        });
static WHITE_RIGHT_CASTLE: Move<'static> =
    Move {
        source: Position {column: 4, row: 0},
        destination: Position {column: 6, row: 0},
        extra_castling_move: &WHITE_RIGHT_ROOK_MOVE,
        enables_en_passant: false,
        en_passant_target: None,
        promotion_piece_type: None,
    };

static BLACK_LEFT_ROOK_MOVE: Option<Move<'static>> =
    Some(Move {
        source: Position {column: 0, row: 7},
        destination: Position {column: 3, row: 0},
        extra_castling_move: &NONE,
        enables_en_passant: false,
        en_passant_target: None,
        promotion_piece_type: None,
    }); static BLACK_LEFT_CASTLE: Move<'static> =
    Move {
        source: Position {column: 4, row: 7},
        destination: Position {column: 2, row: 7},
        extra_castling_move: &BLACK_LEFT_ROOK_MOVE,
        enables_en_passant: false,
        en_passant_target: None,
        promotion_piece_type: None,
    };

static BLACK_RIGHT_ROOK_MOVE: Option<Move<'static>> =
    Some(Move {
        source: Position {column: 7, row: 7},
        destination: Position {column: 5, row: 7},
        extra_castling_move: &NONE,
        enables_en_passant: false,
        en_passant_target: None,
        promotion_piece_type: None,
    });
static BLACK_RIGHT_CASTLE: Move<'static> =
    Move {
        source: Position {column: 4, row: 7},
        destination: Position {column: 6, row: 7},
        extra_castling_move: &BLACK_RIGHT_ROOK_MOVE,
        enables_en_passant: false,
        en_passant_target: None,
        promotion_piece_type: None,
    };

#[derive(Clone)]
pub struct GameState {
    pub current_player: Color,
    board: [[Option<Piece>; 8]; 8],
    // Located here so we don't have to sweep the board of en passant targets after each turn.
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
                board[row][column] = raw_board[row][column].to_piece(true);
            }
        }

        GameState {
            board: board,
            current_player: Color::White,
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
        let game_state = self.clone();
        let player_move = player_brain(&game_state);
        self.move_piece(&player_move);

        self.current_player = if self.current_player == Color::White {
            Color::Black
        } else {
            Color::White
        };
    }

    pub fn get_all_pieces(&self) -> Vec<Piece> {
        let mut result = vec![];
        for col in 0..8 {
            for row in 0..8 {
                if let Some(piece) = self.get_piece(&Position { column: col, row: row }) {
                    result.push(piece);
                }
            }
        }
        
        result
    }

    fn get_piece(&self, position: &Position) -> Option<Piece> {
        self.board[position.row as usize][position.column as usize]
    }

    fn set_piece(&mut self, piece: &Option<Piece>, position: &Position) {
        self.board[position.row as usize][position.column as usize] = *piece;
    }

    pub fn move_piece(&mut self, player_move: &Move) {
        // En passant is only possible for the turn after it was enabled.
        self.en_passant_target = None;

        let mut source_piece = self.get_piece(&player_move.source).unwrap();
        if source_piece.piece_type == PieceType::King || source_piece.piece_type == PieceType::Rook {
            source_piece.can_castle = false;
        }

        let result_piece = match player_move.promotion_piece_type {
            None => source_piece,
            Some(promotion_piece_type) => Piece { piece_type: promotion_piece_type, .. source_piece },
        };

        self.set_piece(&Some(result_piece), &player_move.destination);
        self.set_piece(&None, &player_move.source);

        if let Some(extra_castling_move) = player_move.extra_castling_move.clone() {
            let rook = self.get_piece(&extra_castling_move.source);
            self.set_piece(&rook, &extra_castling_move.destination);
            self.set_piece(&None, &extra_castling_move.source);
        }

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

    fn is_empty(&self, position: &Position) -> bool {
        self.get_piece(position).is_none()
    }

    pub fn get_player_moves(&self, color: Color) -> Vec<Move> {
        let mut moves = vec![];
        for row in 0..8 {
            for column in 0..8 {
                let position = &Position { column: column, row: row };
                let piece = self.get_piece(position);
                if piece.map_or(false, |p| p.color == color) {
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
        let mut moves = vec![];
        
        match piece.piece_type {
            PieceType::Pawn => {

                let (direction, start_row, promotion_row) = match piece.color {
                    Color::White => ( 1, 1, 7),
                    Color::Black => (-1, 6, 0), 
                };
                
                if let Some(forward_one) = self.relative(&source, 0, direction) {
                    if forward_one.row == promotion_row {
                        // TODO: Allow promotion pieces other than the queen.
                        moves.push(Move::promotion(source.clone(), forward_one, PieceType::Queen));
                    } else if self.get_occupation_status(&piece, &forward_one) == OccupationStatus::Empty {
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
                    if left_attack.row == promotion_row {
                        // TODO: Allow promotion pieces other than the queen.
                        moves.push(Move::promotion(source.clone(), left_attack, PieceType::Queen));
                    } else if self.get_occupation_status(&piece, &left_attack) == OccupationStatus::Enemy {
                        moves.push(Move::simple(source.clone(), left_attack));
                    }
                }

                if let Some(right_attack) = self.relative(&source, 1, direction) {
                    if right_attack.row == promotion_row {
                        // TODO: Allow promotion pieces other than the queen.
                        moves.push(Move::promotion(source.clone(), right_attack, PieceType::Queen));
                    } else if self.get_occupation_status(&piece, &right_attack) == OccupationStatus::Enemy {
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
            },

            PieceType::Knight => {
                moves.append(&mut self.get_consecutive_moves(
                        source,
                        &[(-2, -1), (-2, 1), (-1, -2), (-1, 2), (1, -2), (1, 2), (2, -1), (2, 1)],
                        Some(1) 
                        ));
            },

            PieceType::Bishop => {
                moves.append(&mut self.get_consecutive_moves(
                        source,
                        &[(-1, -1), (-1, 1), (1, -1), (1, 1)],
                        None
                        ));
            },

            PieceType::Rook => {
                moves.append(&mut self.get_consecutive_moves(
                        source,
                        &[(-1, 0), (0, -1), (0, 1), (1, 0)],
                        None
                        ));
            },

            PieceType::Queen => {
                moves.append(&mut self.get_consecutive_moves(
                        source,
                        &[(-1,-1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)],
                        None
                        ));

            },
            PieceType::King => {
                moves.append(&mut self.get_consecutive_moves(
                        source,
                        &[(-1,-1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)],
                        Some(1)
                        ));

                if piece.can_castle {
                    let row = if piece.color == Color::White { 0 } else { 7 };
                    let left_rook_position = Position { column: 0, row: row };
                    if self.get_piece(&left_rook_position).map_or(false, |piece| piece.can_castle)
                            && self.are_all_empty(&[(1,row), (2,row), (3,row)]) {
                        moves.push(if piece.color == Color::White {
                            WHITE_LEFT_CASTLE.clone()
                        } else {
                            BLACK_LEFT_CASTLE.clone()
                        });
                    }

                    if self.get_piece(&Position { column: 7, row: row}).map_or(false, |piece| piece.can_castle)
                            && self.are_all_empty(&[(5,row), (6,row)]) {
                        moves.push(if piece.color == Color::White {
                            WHITE_RIGHT_CASTLE.clone()
                        } else {
                            BLACK_RIGHT_CASTLE.clone()
                        });
                    }
                }

            },
        }

        moves
    }

    fn get_consecutive_moves(
            &self,
            source: &Position,
            dirs: &[(i8, i8)],
            max_moves: Option<u8>) -> Vec<Move> {

        let source_piece = &self.get_piece(source).unwrap();

        let mut moves = vec![];

        for &(col_dir, row_dir) in dirs {
            if col_dir == 0 && row_dir == 0 {
                panic!("at least one dir must be non-zero");
            }

            let mut current_dir_moves = vec![];
            let mut dest = source.clone();
            loop {
                dest = dest.relative(col_dir, row_dir);
                if !self.is_in_bounds(&dest)
                        || max_moves.map_or(false, |max| current_dir_moves.len() >= max as usize) {
                    break;
                }

                match self.get_occupation_status(source_piece, &dest) {
                    OccupationStatus::Friendly => break,
                    OccupationStatus::Empty => current_dir_moves.push(Move::simple(source.clone(), dest.clone())),
                    OccupationStatus::Enemy => {
                        current_dir_moves.push(Move::simple(source.clone(), dest.clone()));
                        break;
                    },
                }
            }

            moves.append(&mut current_dir_moves);
        }

        moves
    }

    fn are_all_empty(&self, positions: &[(i8, i8)]) -> bool {
        for position in positions.iter() {
            if !self.is_empty(&Position { column: position.0, row: position.1 }) {
                return false;
            }
        }

        true
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
pub struct Piece {
    pub color: Color,
    pub piece_type: PieceType,
    can_castle: bool,
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
    fn to_piece(&self, can_castle: bool) -> Option<Piece>;
}

impl ToPiece for char {
    fn to_piece(&self, can_castle: bool) -> Option<Piece> {
        if *self == '-' {
            None
        } else {
            let piece_type = match self.to_uppercase().next().unwrap() {
                    'P' => PieceType::Pawn,
                    'N' => PieceType::Knight,
                    'B' => PieceType::Bishop,
                    'R' => PieceType::Rook,
                    'Q' => PieceType::Queen,
                    'K' => PieceType::King,
                    _   => unreachable!()
            };
            let can_castle = if piece_type == PieceType::King || piece_type == PieceType::Rook {
                    can_castle
                } else {
                    false
                };
            Some(Piece {
                color: if self.is_uppercase() { Color::White } else { Color::Black },
                piece_type: piece_type,
                // Only Kings and Rooks can be involved in castling, regardless of what was
                // specified.
                can_castle: can_castle,
            })
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum Color {
    White,
    Black
}
