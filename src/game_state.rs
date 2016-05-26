use std::collections::HashMap;
use std::collections::HashSet;

use piece_type::PieceType;
use position::Position;
use piece_move::Move;
use piece_move::ExtraCastlingMove;

#[derive(Clone)]
pub struct GameState {
    pub current_player: Color,
    board: [[Option<Piece>; 8]; 8],
    // Located here so we don't have to sweep the board of en passant targets after each turn.
    en_passant_target: Option<Position>,
    previous_player_dests: HashSet<Position>,
    pub previous_state_counts: HashMap<String, u8>,
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

        let mut previous_state_counts = HashMap::new();
        previous_state_counts.insert(GameState::custom_hash(board), 1);
        GameState {
            board: board,
            current_player: Color::White,
            en_passant_target: Option::None,
            previous_player_dests: HashSet::new(),
            previous_state_counts: previous_state_counts,
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

        result.push_str(format!("{:?} to play\n", self.current_player).as_str());

        result
    }

    pub fn play_turn(
            &mut self,
            player_brain:
                &Box<Fn(&GameState, &Vec<Move>, u8) -> Move>,
            max_ai_depth: &u8) -> PlayerState {

        let game_state = self.clone();
        let player_state = game_state.get_player_moves();
        if let PlayerState::CanMove(moves) = player_state.clone() {
            let player_move = player_brain(&game_state, &moves, max_ai_depth.clone());
            println!("{:?} played {}", game_state.current_player, player_move.simple_format());
            self.move_piece(&player_move);
        }

        player_state
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

        // Update the map of previous states
        let hash = GameState::custom_hash(self.board);
        if self.previous_state_counts.contains_key(&hash) {
            *self.previous_state_counts.get_mut(&hash).unwrap() += 1;
        } else {
            self.previous_state_counts.insert(hash, 1);
        }

        self.current_player = if self.current_player == Color::White {
            Color::Black
        } else {
            Color::White
        };
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

    // Can't figure out how to get actual Rust hashing working.
    // This function is too simple since it doesn't account for special states like en passant.
    fn custom_hash(board: [[Option<Piece>; 8]; 8]) -> String {
        let mut chars = vec![];
        for i in 0..8 {
            for j in 0..8 {
                chars.push(board[i][j].map_or('-', |p| p.to_char()));
            }
        }

        chars.into_iter().collect::<String>()
    }

    pub fn get_player_moves(&self) -> PlayerState {
        let moves = self.get_player_moves_base(self.current_player).into_iter()
            .filter(|player_move| {
                let mut game_state = self.clone();
                game_state.move_piece(&player_move);
                !game_state.is_in_check(self.current_player)
            }).collect::<Vec<_>>();


        // TODO: Make this visible to the AI rather than it needing to determine it separately.
        if self.previous_state_counts.values().any(|&count| count >= 3) {
            return PlayerState::Stalemate;
        }

        if !moves.is_empty() {
            PlayerState::CanMove(moves)
        } else if self.is_in_check(self.current_player) {
            PlayerState::Checkmate
        } else {
            PlayerState::Stalemate
        }
    }

    pub fn is_in_check(&self, player: Color) -> bool {
        let king_position = match self.find_piece(PieceType::King, player) {
            Some(kp) => kp,
            None => {
                println!("SPECIAL");
                println!("{}", self.format());
                panic!(format!("No king on the board for {:?}!", player));
            }
        };

        self.get_player_moves_without_check(player.opposite()).iter()
            .any(|opponent_move| opponent_move.destination == king_position)
    }

    fn find_piece(&self, piece_type: PieceType, player: Color) -> Option<Position> {
        self.to_vec().into_iter()
            .find(|&(piece, _)| piece.piece_type == piece_type && piece.color == player)
            .map(|(_, position)| position)
    }

    fn to_vec(&self) -> Vec<(Piece, Position)> {
        let mut piece_positions = vec![];
        for row in 0..8 {
            for column in 0..8 {
                if let Some(piece) = self.board[row][column] {
                    piece_positions.push(
                        (piece, Position { column: column as i8, row: row as i8}));
                }
            }
        }

        piece_positions
    }

    pub fn get_player_moves_without_check(&self, color: Color) -> Vec<Move> {
        self.get_player_moves_base(color)
    }

    fn get_player_moves_base(&self, color: Color) -> Vec<Move> {
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
                    if self.get_occupation_status(&piece, &forward_one) == OccupationStatus::Empty {
                        if forward_one.row == promotion_row {
                            // TODO: Allow promotion pieces other than the queen.
                            moves.push(Move::promotion(source.clone(), forward_one, PieceType::Queen));
                        } else {
                            moves.push(Move::simple(source.clone(), forward_one));
                            if let Some(forward_two) = self.relative(source, 0, 2 * direction) {
                                // Pawns can move forward only one unless they are in their starting row.
                                if source.row == start_row &&
                                    self.get_occupation_status(&piece, &forward_two)
                                        == OccupationStatus::Empty {
                                    moves.push(Move::en_passant_enabler(source.clone(), forward_two));
                                }
                            }
                        }
                    }
                }

                // Pawns can take pieces on diagonals immediately in front of them.
                if let Some(left_attack) = self.relative(&source, -1, direction) {
                    if self.get_occupation_status(&piece, &left_attack) == OccupationStatus::Enemy {
                        if left_attack.row == promotion_row {
                            // TODO: Allow promotion pieces other than the queen.
                            moves.push(Move::promotion(source.clone(), left_attack, PieceType::Queen));
                        } else {
                            moves.push(Move::simple(source.clone(), left_attack));
                        }
                    }
                }

                if let Some(right_attack) = self.relative(&source, 1, direction) {
                    if self.get_occupation_status(&piece, &right_attack) == OccupationStatus::Enemy {
                        if right_attack.row == promotion_row {
                            // TODO: Allow promotion pieces other than the queen.
                            moves.push(Move::promotion(source.clone(), right_attack, PieceType::Queen));
                        } else {
                            moves.push(Move::simple(source.clone(), right_attack));
                        }
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

                        let rook_move = ExtraCastlingMove {
                            source: source.relative(-4, 0),
                            destination: source.relative(-1, 0),
                        };
                        moves.push(
                            Move::castle(source.clone(), source.relative(-2, 0), rook_move));
                    }

                    if self.get_piece(&Position { column: 7, row: row}).map_or(false, |piece| piece.can_castle)
                            && self.are_all_empty(&[(5,row), (6,row)]) {
                        let rook_move = ExtraCastlingMove {
                            source: source.relative(3, 0),
                            destination: source.relative(1, 0),
                        };
                        moves.push(
                            Move::castle(source.clone(), source.relative(2, 0), rook_move));
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

    pub fn get_end_state(&self, moves: &Vec<Move>) -> EndState {
        if self.previous_state_counts.values().any(|&count| count >= 3) {
            return EndState::Stalemate;
        }

        if !self.get_all_pieces().iter()
                .any(|piece| piece.color == self.current_player && piece.piece_type == PieceType::King) {
            // The King has been taken.
            return EndState::Win(self.current_player.opposite());
        }

        if moves.is_empty() {
            return EndState::Stalemate;
        }

        EndState::NotEnded
    }
}

#[derive(PartialEq)]
pub enum EndState {
    NotEnded,
    Win(Color),
    Stalemate,
}

#[derive(PartialEq)]
enum OccupationStatus {
    Empty,
    Friendly,
    Enemy,
}

#[derive(Copy, Clone, Hash)]
pub struct Piece {
    pub color: Color,
    pub piece_type: PieceType,
    can_castle: bool,
}

impl Piece {
    fn to_char(&self) -> char {
        let result = match (self.piece_type, self.color) {
            (PieceType::Pawn  , Color::White) => '♟',
            (PieceType::Pawn  , Color::Black) => '♙',
            (PieceType::Knight, Color::White) => '♞',
            (PieceType::Knight, Color::Black) => '♘',
            (PieceType::Bishop, Color::White) => '♝',
            (PieceType::Bishop, Color::Black) => '♗',
            (PieceType::Rook  , Color::White) => '♜',
            (PieceType::Rook  , Color::Black) => '♖',
            (PieceType::Queen , Color::White) => '♛',
            (PieceType::Queen , Color::Black) => '♕',
            (PieceType::King  , Color::White) => '♚',
            (PieceType::King  , Color::Black) => '♔',
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

#[derive(PartialEq, Clone, Copy, Debug, Hash)]
pub enum Color {
    White,
    Black
}

impl Color {
    fn opposite(&self) -> Color {
        match *self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Clone)]
pub enum PlayerState {
    CanMove(Vec<Move>),
    Checkmate,
    Stalemate,
}

