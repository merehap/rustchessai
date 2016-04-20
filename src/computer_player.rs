use std::cmp::Ordering;

use piece_type::PieceType;
use piece_type::PieceType::*;
use piece_move::Move;
use game_state::GameState;
use game_state::Color;

const MAX_DEPTH: u8 = 4;

pub fn max_spaces_comp(initial_game_state: &GameState) -> Option<Move> {
    computer_player(initial_game_state, "MAX SPACES".to_owned(), Box::new(max_spaces_eval))
}

pub fn max_moves_comp(initial_game_state: &GameState) -> Option<Move> {
    computer_player(initial_game_state, "MAX MOVES".to_owned(), Box::new(max_moves_eval))
}

pub fn piece_score_comp(initial_game_state: &GameState) -> Option<Move> {
    computer_player(initial_game_state, "PIECE SCORE".to_owned(), Box::new(piece_score_eval))
}

fn computer_player<'a>(
        initial_game_state: &'a GameState,
        name: String,
        eval_function: Box<Fn(&GameState) -> i16>)
        -> Option<Move<'a>> {

    if let Some((best_move, best_score)) = determine_best_move(initial_game_state, &eval_function, MAX_DEPTH) {
        println!("Current score according to the {} AI ({:?}): {}",
            name,
            initial_game_state.current_player,
            best_score);
        return Some(best_move);
    }

    None
}

fn determine_best_move<'a>(
        initial_game_state: &'a GameState,
        eval_function: &Box<Fn(&GameState) -> i16>,
        ply: u8)
        -> Option<(Move<'a>, i16)> {

    if ply == 0 {
        return None;
    }

    let moves = initial_game_state.get_player_moves(initial_game_state.current_player);
    if moves.is_empty() {
        return None;
    }

    let current_player = initial_game_state.current_player;
    let mut best_score = if current_player == Color::White { i16::min_value() } else { i16::max_value() };

    let mut best_move = moves[0].clone();
    for piece_move in moves {
        let mut game_state = initial_game_state.clone();
        game_state.move_piece(&piece_move);
        let score = if ply == 1 {
            // Use the base, non-recursive heuristic if we are only looking ahead one move.
            eval_function(&game_state)
        } else {
            // Determine the other player's best move, returning 0 if there isn't a move,
            // indicating stalemate.
            determine_best_move(&game_state, eval_function, ply - 1)
                .map_or(0, |(_, s)| s)
        };

        if (current_player == Color::White && score > best_score)
                || (current_player == Color::Black && score < best_score) {
            best_score = score;
            best_move = piece_move.clone();
        }
    }

    Some((best_move, best_score))
}

fn max_moves_eval(game_state: &GameState) -> i16 {
    let piece_score = piece_score_eval(&game_state); 

    let move_score = game_state.get_player_moves(Color::White).len() as i16
            - game_state.get_player_moves(Color::Black).len() as i16;

    piece_score + move_score as i16
}

fn max_spaces_eval(game_state: &GameState) -> i16 {
    let piece_score = piece_score_eval(&game_state);

    let mut ownership_grid = [[0; 8]; 8];

    for white_move in game_state.get_player_moves(Color::White) {
        let dest = white_move.destination.clone();
        ownership_grid[dest.row as usize][dest.column as usize] += 1;
    }

    for white_move in game_state.get_player_moves(Color::Black) {
        let dest = white_move.destination.clone();
        ownership_grid[dest.row as usize][dest.column as usize] -= 1;
    }

    let mut space_score = 0;
    for col in 0..8 {
        for row in 0..8 {
            match ownership_grid[row][col].cmp(&0) {
                Ordering::Less => space_score -= 1,
                Ordering::Greater => space_score += 1,
                _ => (),
            }
        }
    }

    piece_score + space_score
}

fn piece_score_eval(game_state: &GameState) -> i16 {
    15 * game_state.get_all_pieces().iter().map(|piece| {
        let sign = if piece.color == Color::White { 1 } else { -1 };
        sign * piece_value(&piece.piece_type) as i16
    }).fold(0, |x, y| x + y)
}

fn piece_value(piece_type: &PieceType) -> i8 {
    match *piece_type {
        Pawn => 1,
        Knight => 3,
        Bishop => 3,
        Rook => 5,
        Queen => 9,
        King => 127,
    }
}
