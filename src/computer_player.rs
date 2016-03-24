use piece_type::PieceType;
use piece_type::PieceType::*;
use piece_move::Move;
use game_state::GameState;
use game_state::Color;

pub fn computer_player(initial_game_state: &GameState) -> Move {
    let moves = initial_game_state.get_player_moves(initial_game_state.current_player);
    if moves.is_empty() {
        panic!("no moves possible!");
    }

    let current_player = initial_game_state.current_player;
    let mut best_score = if current_player == Color::White { i16::min_value() } else { i16::max_value() };

    let mut best_move = moves[0].clone();
    for piece_move in moves {
        let mut game_state = initial_game_state.clone();
        game_state.move_piece(&piece_move);
        let score = count_current_score(&game_state);
        if (current_player == Color::White && score > best_score)
                || (current_player == Color::Black && score < best_score) {
            best_score = score;
            best_move = piece_move.clone();
        }
    }

    best_move
}

fn count_current_score(game_state: &GameState) -> i16 {
    let piece_score = 10 * game_state.get_all_pieces().iter().map(|piece| {
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
