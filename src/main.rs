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
use computer_player::computer_player;

fn main() {
    let mut game_state = GameState::opening_state();
    loop {
        println!("{}", game_state.format());
        game_state.play_turn(Box::new(human_player));
        println!("{}", game_state.format());
        game_state.play_turn(Box::new(computer_player));
    }
}
