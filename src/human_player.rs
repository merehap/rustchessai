use std::io;

use piece_move::Move;
use game_state::GameState;

pub fn human_player(game_state: &GameState) -> Option<Move> {
    let moves = game_state.get_player_moves(game_state.current_player);
    loop {
        println!("Enter a move:");
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input = input.trim_right_matches('\n').to_owned();
        match Move::from_notation(&input) {
            None => println!("Invalid move"),
            Some(player_move) => {
                match moves.iter().find(|m| m.source == player_move.source
                                        && m.destination == player_move.destination) {
                    None => println!("Illegal move"),
                    Some(result) => {
                        return Some(result.clone());
                    }
                }
            },
        };
    }
}
