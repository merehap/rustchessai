#![feature(plugin)]
#![plugin(clippy)]

use std::collections::HashSet;
use std::io;

fn main() {
    let mut game_state = GameState::opening_state();
    loop {
        println!("{}", game_state.format());
        game_state.play_turn(Box::new(human_player));
    }
}

fn initial_player_state() -> PlayerState {
    PlayerState {
        can_castle_short: true,
        can_castle_long: true,
    }
}

#[derive(Clone)]
struct GameState {
    board: [[Option<Piece>; 8]; 8],
    current_player: Color,
    white_state: PlayerState,
    black_state: PlayerState,
}

impl GameState {
    fn opening_state() -> GameState {
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
        }
    }

    fn format(&self) -> String {
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

    fn play_turn(&mut self, player_brain: Box<Fn(&GameState) -> Move>) {
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

    fn move_piece(&mut self, player_move: &Move) {
        self.board[player_move.destination.row as usize][player_move.destination.column as usize] =
            self.get_piece(&player_move.source);
        self.board[player_move.source.row as usize][player_move.source.column as usize] = Option::None;
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

    fn get_current_player_moves(&self) -> Vec<Move> {
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
                let direction = if piece.color == Color::White { 1 } else { -1 };
                let start_row = if piece.color == Color::White { 1 } else {  6 };

                // Pawns can move forward only one unless they are in their starting row.
                let max_moves = if source.row == start_row { 2 } else { 1 };
                dests.append(&mut self.get_consecutive_dests(
                        source,
                        &[(0, direction)],
                        Some(max_moves),
                        // Pawns can't take when moving forward.
                        StopOption::BeforeEnemy
                        ));

                // Pawns can take pieces on diagonals immediately in front of them.
                dests.append(&mut self.get_consecutive_dests(
                        source,
                        &[(-1, direction), (1, direction)],
                        Some(1),
                        StopOption::OnEmpty
                        ));

                // TODO: Add en passant
            },

            PieceType::Knight => {
                dests.append(&mut self.get_custom_dests(
                        source,
                        &[(-2, -1), (-2, 1), (-1, -2), (-1, 2), (1, -2), (1, 2), (2, -1), (2, 1)],
                        ));
            },

            PieceType::Bishop => {
                dests.append(&mut self.get_consecutive_dests(
                        source,
                        &[(-1, -1), (-1, 1), (1, -1), (1, 1)],
                        None,
                        StopOption::OnEnemy
                        ));
            },

            PieceType::Rook => {
                dests.append(&mut self.get_consecutive_dests(
                        source,
                        &[(-1, 0), (0, -1), (0, 1), (1, 0)],
                        None,
                        StopOption::OnEnemy
                        ));
            },

            PieceType::Queen => {
                dests.append(&mut self.get_consecutive_dests(
                        source,
                        &[(-1,-1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)],
                        None,
                        StopOption::OnEnemy
                        ));

            },
            PieceType::King => {
                dests.append(&mut self.get_consecutive_dests(
                        source,
                        &[(-1,-1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)],
                        Some(1),
                        StopOption::OnEnemy
                        ));

                //TODO: Add castling.
            },
        }

        dests.iter().map(|dest| Move { source: source.clone(), destination: dest.clone() }).collect::<Vec<_>>()
    }

    fn get_consecutive_dests(
            &self,
            source: &Position,
            dirs: &[(i8, i8)],
            max_moves: Option<u8>,
            take_option: StopOption) -> Vec<Position> {

        let source_piece = &self.get_piece(source).unwrap();

        let mut dests = vec![];

        for &(col_dir, row_dir) in dirs {
            if col_dir == 0 && row_dir == 0 {
                panic!("at least one dir must be non-zero");
            }

            if col_dir.abs() > 1 || row_dir.abs() > 1 {
                panic!("dirs must be between -1 and 1");
            }

            let mut dest = source.clone();
            loop {
                dest = dest.relative(col_dir, row_dir);
                if !self.is_in_bounds(&dest) || max_moves.map_or(false, |max| dests.len() >= max as usize) {
                    break;
                }
                
                match (self.get_occupation_status(source_piece, &dest), take_option) {
                    (OccupationStatus::Empty   , StopOption::OnEmpty ) => break,
                    (OccupationStatus::Empty   , _                    ) => dests.push(dest.clone()),
                    (OccupationStatus::Enemy   , StopOption::OnEmpty ) => dests.push(dest.clone()),
                    (OccupationStatus::Enemy   , StopOption::OnEnemy  ) => { dests.push(dest.clone()); break; },
                    (_                         , _                    ) => break,
                }
            }
        }

        dests
    }

    fn get_custom_dests(&self, source: &Position, relative_dests: &[(i8, i8)]) -> Vec<Position> {
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

#[derive(PartialEq, Clone, Copy)]
enum StopOption {
    OnEnemy,
    BeforeEnemy,
    OnEmpty,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Position {
    column: i8,
    row: i8,
}

impl Position {
    fn from_notation(notation: &str) -> Option<Position> {
        if notation.len() != 2 {
            return None;
        }

        let mut chars = notation.chars();
        let col = match chars.next().unwrap() {
                'a' => Some(0),
                'b' => Some(1),
                'c' => Some(2),
                'd' => Some(3),
                'e' => Some(4),
                'f' => Some(5),
                'g' => Some(6),
                'h' => Some(7),
                _   => None,
        };

        if col.is_none() {
            return None;
        }

        let row = chars.next().unwrap().to_digit(10).unwrap();
        if row == 0 || row > 8 {
            return None;
        }

        Some(Position {
            column: col.unwrap(),
            row: (row - 1) as i8,
        })
    }

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

#[derive(Debug, PartialEq, Eq, Hash)]
struct Move {
    source: Position,
    destination: Position,
}

impl Move {

    fn from_notation(notation: &str) -> Option<Move> {
        if notation.len() != 4 {
            return None;
        }

        let source = Position::from_notation(&notation[0..2].to_owned());
        let dest = Position::from_notation(&notation[2..4].to_owned());
        if source.is_none() || dest.is_none() {
            return None;
        }

        Some (Move {
            source: source.unwrap(),
            destination: dest.unwrap(),
        })
    }

    fn format(&self, game_state: &GameState) -> String {
        let piece = game_state.get_piece(&self.source).unwrap();
        let piece_text: String = if piece.piece_type == PieceType::Pawn {
            "".to_owned()
        } else {
            format!("{}", piece.to_char().to_uppercase().next().unwrap())
        };

        let move_type_text = if game_state.get_piece(&self.destination).is_some() {"x"} else {""};
        
        format!("{}{}{}", piece_text, move_type_text, self.destination.format())
    }
}

#[derive(Clone)]
struct PlayerState {
    can_castle_short: bool,
    can_castle_long: bool,
    // Add en passant moves.
}

fn human_player(game_state: &GameState) -> Move {
    let moves = game_state.get_current_player_moves();
    let move_set = moves.iter().collect::<HashSet<_>>();
    loop {
        println!("Enter a move:");
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input = input.trim_right_matches('\n').to_owned();
        match Move::from_notation(&input) {
            None => println!("Invalid move"),
            Some(player_move) => {
                if move_set.contains(&player_move) {
                    return player_move;
                }

                println!("Illegal move");
            },
        };
    }
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
