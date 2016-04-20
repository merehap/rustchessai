#![feature(plugin)]
#![plugin(clippy)]

mod piece_type;
mod position;
mod piece_move;
mod game_state;
mod human_player;
mod computer_player;

use std::collections::HashMap;

use game_state::GameState;
use human_player::human_player;
use computer_player::max_moves_comp;
use computer_player::max_spaces_comp;
use piece_move::Move;

fn main() {
    let mut players: HashMap<String, Box<Fn(&GameState) -> Option<Move>>> = HashMap::new();
    players.insert("human".to_owned(), Box::new(human_player));
    players.insert("max_moves".to_owned(), Box::new(max_moves_comp));
    players.insert("max_spaces".to_owned(), Box::new(max_spaces_comp));

    let stdin = std::io::stdin();

    let mut player_1_text = String::new();
    println!("What should player 1 be? Options: {:?}", players.keys().collect::<Vec<_>>());
    stdin.read_line(&mut player_1_text).unwrap();
    let player_1 = &players[&player_1_text.trim().to_owned()];

    let mut player_2_text = String::new();
    println!("What should player 2 be? Options: {:?}", players.keys().collect::<Vec<_>>());
    stdin.read_line(&mut player_2_text).unwrap();
    let player_2 = &players[&player_2_text.trim().to_owned()];

    let mut game_state = GameState::opening_state();
    loop {
        println!("{}", game_state.format());
        if !game_state.play_turn(player_1) {
            println!("Black won!");
            return;
        }

        println!("{}", game_state.format());
        if !game_state.play_turn(player_2) {
            println!("White won!");
            return;
        }
    }
}
