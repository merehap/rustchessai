use std::io;

use piece_move::Move;
use game_state::GameState;

pub fn human_player(_: &GameState, moves: &Vec<Move>, _: u8) -> Move {
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
                        return result.clone();
                    }
                }
            },
        };
    }
}
