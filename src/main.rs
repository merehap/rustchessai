#![feature(plugin)]
#![plugin(clippy)]

mod piece_type;
mod position;
mod piece_move;
mod game_state;
mod human_player;
mod computer_player;

use game_state::GameState;
use human_player::human_player;
use computer_player::max_moves_comp;
use computer_player::max_spaces_comp;

fn main() {
    let mut game_state = GameState::opening_state();
    loop {
        println!("{}", game_state.format());
        if !game_state.play_turn(Box::new(max_moves_comp)) {
            println!("Black won!");
            return;
        }

        println!("{}", game_state.format());
        if !game_state.play_turn(Box::new(max_spaces_comp)) {
            println!("White won!");
            return;
        }
    }
}
