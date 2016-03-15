fn main() {
    let mut game_state = initial_game_state();
    println!("{}", game_state.format());
    for player_move in game_state.get_all_moves_for_player(Color::White) {
        println!("{:?}", player_move.format(&game_state));
    }
}

fn initial_game_state() -> GameState {
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
        white_state: initial_player_state(),
        black_state: initial_player_state(),
    }
}

fn initial_player_state() -> PlayerState {
    PlayerState {
        can_castle_short: true,
        can_castle_long: true,
    }

}

struct GameState {
    board: [[Option<Piece>; 8]; 8],
    white_state: PlayerState,
    black_state: PlayerState,
}

impl GameState {
    fn format(&self) -> String {
        let mut result: String = String::new();
        for row in 0..8 {
            for column in 0..8 {
                result.push(self.board[row][column].map_or('-', |piece| piece.to_char()));
                result.push(' ');
            }

            result.push('\n');
        }

        result
    }

    fn get_piece(&self, position: &Position) -> Option<Piece> {
        self.board[position.row as usize][position.column as usize]
    }

    fn is_occupied(&self, position: &Position) -> bool {
        self.get_piece(position).is_some()
    }

    fn is_empty(&self, position: &Position) -> bool {
        self.get_piece(position).is_none()
    }

    fn is_in_bounds(&self, position: &Position) -> bool {
        position.column >= 0 && position.row >= 0 && position.column < 8 && position.row < 8
    }

    fn get_occupation_status(&self, friendly_piece: &Piece, dest: &Position) -> OccupationStatus {
        match self.get_piece(dest) {
            None => OccupationStatus::Empty,
            Some(piece) if piece.color == friendly_piece.color => OccupationStatus::Friendly,
            _ => OccupationStatus::Enemy,
        }
    }

    fn get_all_moves_for_player(&self, color: Color) -> Vec<Move> {
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
        let mut dests = vec![];
        
        match piece.piece_type {
            PieceType::Pawn => {
                let direction = if piece.color == Color::White { 1 } else { -1 };
                let start_row = if piece.color == Color::White { 1 } else {  6 };

                let max_moves = if source.row == start_row { 2 } else { 1 };
                dests.append(&mut self.get_consecutive_dests(
                        source, 0, direction, Some(max_moves), &TakeOption::CannotTake));

                dests.append(&mut self.get_consecutive_dests(
                        source, -1, direction, Some(1), &TakeOption::OnlyTake));
                dests.append(&mut self.get_consecutive_dests(
                        source,  1, direction, Some(1), &TakeOption::OnlyTake));

                // TODO: Add en passant
            },

            PieceType::Knight => {
                dests.append(&mut self.get_custom_dests(
                    source,
                    &vec![(-2, -1), (-2, 1), (-1, -2), (-1, 2), (1, -2), (1, 2), (2, -1), (2, 1)]));
            }

            _ => {}
        }

        dests.iter().map(|dest| Move { source: source.clone(), destination: dest.clone() }).collect::<Vec<_>>()
    }

    fn get_consecutive_dests(
            &self,
            source: &Position,
            col_dir: i8,
            row_dir: i8,
            max_moves: Option<u8>,
            take_option: &TakeOption) -> Vec<Position> {

        if col_dir == 0 && row_dir == 0 {
            panic!("at least one dir must be non-zero");
        }

        if col_dir.abs() > 1 || row_dir.abs() > 1 {
            panic!("dirs must be between -1 and 1");
        }

        let source_piece = &self.get_piece(source).unwrap();

        let mut moves = vec![];
        let mut dest = source.clone();
        loop {
            dest = dest.relative(col_dir, row_dir);
            if !self.is_in_bounds(&dest) || max_moves.map_or(false, |max| moves.len() >= max as usize) {
                break;
            }
            
            match (self.get_occupation_status(source_piece, &dest), take_option.clone()) {
                (OccupationStatus::Friendly, _                     ) => { break; },
                (OccupationStatus::Empty   , TakeOption::OnlyTake  ) => { break; }
                (OccupationStatus::Empty   , _                     ) => { moves.push(dest.clone()) }
                (OccupationStatus::Enemy   , TakeOption::CanTake   ) => { moves.push(dest.clone()); break; },
                (OccupationStatus::Enemy   , TakeOption::CannotTake) => { break; },
                (OccupationStatus::Enemy   , TakeOption::OnlyTake  ) => { moves.push(dest.clone()); },
            }
        }

        moves
    }

    fn get_custom_dests(&self, source: &Position, relative_dests: &Vec<(i8, i8)>) -> Vec<Position> {
        let source_piece = &self.get_piece(source).unwrap();
        let mut result = vec![];
        for relative_dest in relative_dests {
            let dest = &source.relative(relative_dest.0, relative_dest.1);
            if !self.is_in_bounds(dest) {
                continue;
            }

            if self.get_occupation_status(source_piece, &dest) != OccupationStatus::Friendly {
                result.push(dest.clone());
            }
        }

        result
    }
}

#[derive(PartialEq)]
enum OccupationStatus {
    Empty,
    Friendly,
    Enemy,
}

#[derive(PartialEq, Clone)]
enum TakeOption {
    CanTake,
    OnlyTake,
    CannotTake,
}

#[derive(Clone)]
struct Position {
    column: i8,
    row: i8,
}

impl Position {
    fn relative(&self, column_offset: i8, row_offset: i8) -> Position {
        Position { column: self.column + column_offset, row: self.row + row_offset }
    }

    fn format(&self) -> String {
        let column_text = match self.column {
            0 => "a",
            1 => "b",
            2 => "c",
            3 => "d",
            4 => "e",
            5 => "f",
            6 => "g",
            7 => "h",
            _ => panic!("column must be between 0 and 7 inclusive")
        };

        let row_text = (self.row + 1).to_string();

        format!("{}{}", column_text, row_text)
    }
}

struct Move {
    source: Position,
    destination: Position,
}

impl Move {
    fn format(&self, game_state: &GameState) -> String {
        let piece = game_state.get_piece(&self.source).unwrap();
        let piece_text: String = if piece.piece_type == PieceType::Pawn {
            "".to_string()
        } else {
            format!("{}", piece.to_char().to_uppercase().next().unwrap())
        };

        let move_type_text = if game_state.get_piece(&self.destination).is_some() {"x"} else {""};
        
        format!("{}{}{}", piece_text, move_type_text, self.destination.format())
    }
}

struct PlayerState {
    can_castle_short: bool,
    can_castle_long: bool,
    // Add en passant moves.
}

#[derive(Copy, Clone)]
struct Piece {
    color: Color,
    piece_type: PieceType,
}

impl Piece {
    fn to_i8(&self) -> i8 {
        let mut bits: i8 = 0;

        if self.color == Color::Black {
            bits |= 64;
        }

        bits |= match self.piece_type {
            PieceType::Pawn => 0,
            PieceType::Knight => 1,
            PieceType::Bishop => 2,
            PieceType::Rook => 3,
            PieceType::Queen => 4,
            PieceType::King => 5,
        };

        bits
    }

    fn to_char(&self) -> char {
        match (self.color.clone(), self.piece_type.clone()) {
            (Color::White, PieceType::Pawn) => 'P',
            (Color::Black, PieceType::Pawn) => 'p',
            (Color::White, PieceType::Knight) => 'N',
            (Color::Black, PieceType::Knight) => 'n',
            (Color::White, PieceType::Bishop) => 'B',
            (Color::Black, PieceType::Bishop) => 'b',
            (Color::White, PieceType::Rook) => 'R',
            (Color::Black, PieceType::Rook) => 'r',
            (Color::White, PieceType::Queen) => 'Q',
            (Color::Black, PieceType::Queen) => 'q',
            (Color::White, PieceType::King) => 'K',
            (Color::Black, PieceType::King) => 'k',
        }
    }
}

trait ToPiece {
    fn to_piece(&self) -> Option<Piece>;
}

impl ToPiece for u8 {
    fn to_piece(&self) -> Option<Piece> {
        if (self & 128) != 0 {
            None
        } else {
            Some (Piece {
                color: if (self & 64) == 0 { Color::White } else { Color::Black },
                piece_type: match self & 7 {
                    0 => PieceType::Pawn,
                    1 => PieceType::Knight,
                    2 => PieceType::Bishop,
                    3 => PieceType::Rook,
                    4 => PieceType::Queen,
                    5 => PieceType::King,
                    _ => unreachable!()
                }
            })
        }
    }
}

impl ToPiece for char {
    fn to_piece(&self) -> Option<Piece> {
        if self.clone() == '-' {
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
