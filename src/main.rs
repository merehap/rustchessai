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
    let mut players: HashMap<String, Box<Fn(&GameState, &Vec<Move>, u8) -> Move>> = HashMap::new();

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

    let mut max_ai_depth_text = String::new();
    println!("Max AI Depth?");
    stdin.read_line(&mut max_ai_depth_text).unwrap();
    let ref max_ai_depth = max_ai_depth_text.trim().parse().unwrap(); 

    let ref mode = modes[&mode_text.trim().to_owned()]; 
    match *mode {
        GameMode::SingleGame => {
            players.insert("human".to_owned(), Box::new(human_player));
            play_single_game(players, max_ai_depth)
        },
        GameMode::AIRoundRobin => {
            println!("How many rounds per match?");
            let mut rounds_per_match = String::new();
            stdin.read_line(&mut rounds_per_match).unwrap();
            play_ai_round_robin(players, rounds_per_match.trim().parse().unwrap(), max_ai_depth);
        }
    }
}

enum GameMode {
    SingleGame,
    AIRoundRobin,
}

fn play_single_game(
        players: HashMap<String, Box<Fn(&GameState, &Vec<Move>, u8) -> Move>>,
        max_ai_depth: &u8) {

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
    
    play_game(player_1, player_2, max_ai_depth);
}

fn play_ai_round_robin(
        players: HashMap<String, Box<Fn(&GameState, &Vec<Move>, u8) -> Move>>,
        rounds_per_match: u8,
        max_ai_depth: &u8) {

    let mut results = [[(0f32, 0f32, 0f32); AI_COUNT]; AI_COUNT];
    for _ in 0..rounds_per_match {
        for i in 0..AI_COUNT {
            for j in 0..AI_COUNT {
                if i == j {
                    continue;
                }

                let ref white = players[players.keys().collect::<Vec<_>>()[i]];
                let ref black = players[players.keys().collect::<Vec<_>>()[j]];
                match play_game(&white, &black, max_ai_depth) {
                    GameResult::WhiteWon => results[i][j].0 += 1.0,
                    GameResult::BlackWon => results[i][j].1 += 1.0,
                    GameResult::Draw     => results[i][j].2 += 1.0,
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
        let mut sum = 0f32;
        for i in 0..AI_COUNT {
            if i == j {
                print!("{cell:>width$}", cell="-", width=width);
            } else {
                let cell = (results[i][j].0 - results[i][j].2) / rounds_per_match as f32;
                sum += cell;
                print!("{cell:>width$.2}", cell=cell, width=width);
            }
        }

        // Cumulative sum for the current black player.
        println!("{cell:>width$.2}",
                 cell=sum as f32/ (rounds_per_match as usize * (AI_COUNT - 1)) as f32,
                 width=width);
    }

    println!("{empty:>width$}", empty="", width=width*(AI_COUNT+1));
    print!("{text:>width$}", text="CUMULATIVE", width=width);
    for i in 0..AI_COUNT {
        let mut sum = 0f32;
        for j in 0..AI_COUNT {
            sum += results[i][j].0;
            sum -= results[i][j].2;
        }

        print!("{cell:>width$.2}",
               cell=sum as f32 / (rounds_per_match as usize * (AI_COUNT - 1)) as f32,
               width=width);
    }

    println!("");
}

fn play_game(
        white: &Box<Fn(&GameState, &Vec<Move>, u8) -> Move>,
        black: &Box<Fn(&GameState, &Vec<Move>, u8) -> Move>,
        max_ai_depth: &u8) -> GameResult {

    let mut game_state = GameState::opening_state();
    let mut turn = 1;
    let game_result;

    loop {
        println!("Turn {}", turn);
        println!("{}", game_state.format());
        match game_state.play_turn(white, max_ai_depth) {
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
        match game_state.play_turn(black, max_ai_depth) {
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
