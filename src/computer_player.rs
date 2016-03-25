use piece_type::PieceType;
use piece_type::PieceType::*;
use piece_move::Move;
use game_state::GameState;
use game_state::Color;

pub fn max_moves_comp<'a>(initial_game_state: &'a GameState) -> Option<Move<'a>> {
    computer_player(initial_game_state, "MAX MOVES".to_owned(), Box::new(max_moves_eval))
}

fn computer_player<'a>(
        initial_game_state: &'a GameState,
        name: String,
        eval_function: Box<Fn(&GameState) -> i16>)
        -> Option<Move<'a>> {

    if let Some((best_move, best_score)) = determine_best_move(initial_game_state, &eval_function, 4) {
        println!("Current score according to the {} AI: {}", name, best_score);
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
        let before_player = game_state.current_player;
        game_state.move_piece(&piece_move);
        let after_player = game_state.current_player;
        let score = if ply == 1 {
            // Use the base, non-recursive heuristic if we are only looking ahead one move.
            eval_function(&game_state)
        } else {
            // Determine the other player's best move, returning 0 if there isn't a move,
            // indicating stalemate.
            determine_best_move(&game_state, eval_function, ply - 1).map_or(0, |(_, s)| s)
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
    let piece_score = 15 * game_state.get_all_pieces().iter().map(|piece| {
        let sign = if piece.color == Color::White { 1 } else { -1 };
        sign * piece_value(&piece.piece_type) as i16
    }).fold(0, |x, y| x + y); 

    let move_score = game_state.get_player_moves(Color::White).len() as i16
            - game_state.get_player_moves(Color::Black).len() as i16;

    piece_score + move_score as i16
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
