#![feature(plugin)]
#![plugin(clippy)]

mod piece_type;
mod position;
mod piece_move;
mod game_state;
mod human_player;

use game_state::GameState;
use human_player::human_player;

fn main() {
    let mut game_state = GameState::opening_state();
    loop {
        println!("{}", game_state.format());
        game_state.play_turn(Box::new(human_player));
    }
}
