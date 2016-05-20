#![feature(advanced_slice_patterns, slice_patterns)]

extern crate rand;

mod piece_type;
mod position;
mod piece_move;
mod game_state;
mod human_player;
mod computer_player;

use std::collections::HashMap;

use game_state::GameState;
use game_state::PlayerState;
use human_player::human_player;
use computer_player::piece_score_comp;
use computer_player::max_moves_comp;
use computer_player::max_spaces_comp;
use computer_player::spaces_moves_comp;
use piece_move::Move;

// TODO Determine this from the players HashMap.
const AI_COUNT: usize = 4;

fn main() {
    let mut players: HashMap<String, Box<Fn(&GameState, &Vec<Move>) -> Move>> = HashMap::new();

    // TODO Unify name as seen here with the value in computer_player.rs.
    players.insert("piece_score".to_owned(), Box::new(piece_score_comp));
    players.insert("max_moves".to_owned(), Box::new(max_moves_comp));
    players.insert("max_spaces".to_owned(), Box::new(max_spaces_comp));
    players.insert("spaces_moves".to_owned(), Box::new(spaces_moves_comp));

    let mut modes = HashMap::new();
    modes.insert("single".to_owned(), GameMode::SingleGame);
    modes.insert("AIs".to_owned(), GameMode::AIRoundRobin);

    let stdin = std::io::stdin();
    let mut mode_text = String::new();
    println!("Mode? Options: {:?}", modes.keys().collect::<Vec<_>>());
    stdin.read_line(&mut mode_text).unwrap();
    let ref mode = modes[&mode_text.trim().to_owned()]; 
    match *mode {
        GameMode::SingleGame => {
            players.insert("human".to_owned(), Box::new(human_player));
            play_single_game(players)
        },
        GameMode::AIRoundRobin => {
            println!("How many rounds per match?");
            let mut rounds_per_match = String::new();
            stdin.read_line(&mut rounds_per_match).unwrap();
            play_ai_round_robin(players, rounds_per_match.trim().parse().unwrap());
        }
    }
}

enum GameMode {
    SingleGame,
    AIRoundRobin,
}

fn play_single_game(players: HashMap<String, Box<Fn(&GameState, &Vec<Move>) -> Move>>) {
    let stdin = std::io::stdin();
    let mut player_1_text = String::new();
    println!("What should player 1 be? Options: {:?}", players.keys().collect::<Vec<_>>());
    stdin.read_line(&mut player_1_text).unwrap();
    println!("{} player chosen.", &player_1_text.trim());
    let ref player_1 = players[&player_1_text.trim().to_owned()];

    let mut player_2_text = String::new();
    println!("What should player 2 be? Options: {:?}", players.keys().collect::<Vec<_>>());
    stdin.read_line(&mut player_2_text).unwrap();
    println!("{} player chosen.", &player_2_text.trim());
    let ref player_2 = players[&player_2_text.trim().to_owned()];
    
    play_game(player_1, player_2);
}

fn play_ai_round_robin(
        players: HashMap<String, Box<Fn(&GameState, &Vec<Move>) -> Move>>,
        rounds_per_match: u8) {

    let mut results = [[0f32; AI_COUNT]; AI_COUNT];
    for _ in 0..rounds_per_match {
        for i in 0..AI_COUNT {
            for j in 0..AI_COUNT {
                let ref white = players[players.keys().collect::<Vec<_>>()[i]];
                let ref black = players[players.keys().collect::<Vec<_>>()[j]];
                results[i][j] += match play_game(&white, &black) {
                    GameResult::WhiteWon => 1f32,
                    GameResult::BlackWon => -1f32,
                    GameResult::Draw     => 0f32,
                };
            }
        }
    }

    // TODO:
    // Display overall win and draw percentages
    // Display per AI win and draw percentages
    // AI ranking as white and black
    let width = 16;

    println!("{white_text:>width$}", white_text="WHITE PLAYER", width=3*width+width/2);
    print!("{empty:>width$}", empty="", width=width);
    for i in 0..AI_COUNT {
        print!("{column:>width$}", column=players.keys().collect::<Vec<_>>()[i], width=width);
    }

    println!("");
    for j in 0..AI_COUNT {
        print!("{row:>width$}", row=players.keys().collect::<Vec<_>>()[j], width=width);
        for i in 0..AI_COUNT {
            print!("{cell:>width$.2}", cell=results[i][j] / rounds_per_match as f32, width=width);
        }

        println!("");
    }

    println!("{empty:>width$}", empty="", width=width*(AI_COUNT+1));
    print!("{text:>width$}", text="CUMULATIVE", width=width);
    for i in 0..AI_COUNT {
        let mut sum = 0f32;
        for j in 0..AI_COUNT {
            sum += results[i][j];
        }

        print!("{cell:>width$}", cell=sum as f32 / AI_COUNT as f32, width=width);
    }

    println!("");
}

fn play_game(
        white: &Box<Fn(&GameState, &Vec<Move>) -> Move>,
        black: &Box<Fn(&GameState, &Vec<Move>) -> Move>) -> GameResult {

    let mut game_state = GameState::opening_state();
    let mut turn = 1;
    let game_result;

    loop {
        println!("Turn {}", turn);
        println!("{}", game_state.format());
        match game_state.play_turn(white) {
            PlayerState::Stalemate => {
                game_result = GameResult::Draw; 
                println!("Draw!");
                break;
            },
            PlayerState::Checkmate => {
                game_result = GameResult::BlackWon; 
                println!("Black won!");
                break;
            },
            _ => (),
        };

        println!("{}", game_state.format());
        match game_state.play_turn(black) {
            PlayerState::Stalemate => {
                game_result = GameResult::Draw; 
                println!("Draw!");
                break; 
            },
            PlayerState::Checkmate => {
                game_result = GameResult::WhiteWon; 
                println!("White won!");
                break;
            },
            _ => (),
        }

        // TODO Replace this with the 50 moves rule.
        if turn == 200 {
            game_result = GameResult::Draw;
            println!("Draw by hitting max moves!");
            break;
        }

        turn += 1;
    }

    println!("Game ended on turn {} .", turn);
    game_result
}

enum GameResult {
    WhiteWon,
    BlackWon,
    Draw,
}
