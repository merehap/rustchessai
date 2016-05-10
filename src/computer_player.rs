use std::cmp::Ordering;

use piece_type::PieceType;
use piece_type::PieceType::*;
use piece_move::Move;
use game_state::GameState;
use game_state::Color;
use game_state::EndState;

const MAX_DEPTH: u8 = 4;

pub fn max_spaces_comp(initial_game_state: &GameState, moves: &Vec<Move>) -> Move {
    computer_player(initial_game_state, &moves, "MAX SPACES".to_owned(),
        Box::new(|game_state| multi_eval(game_state,
            &[(15, &piece_scorer()), (3, &spaces_scorer())])))
}

pub fn max_moves_comp(initial_game_state: &GameState, moves: &Vec<Move>) -> Move {
    computer_player(initial_game_state, &moves, "MAX MOVES".to_owned(),
        Box::new(|game_state| multi_eval(game_state,
            &[(15, &piece_scorer()), (1, &moves_scorer())])))
}

pub fn piece_score_comp(initial_game_state: &GameState, moves: &Vec<Move>) -> Move {
    computer_player(initial_game_state, &moves, "PIECE SCORE".to_owned(),
        Box::new(|game_state| multi_eval(game_state,
            &[(15, &piece_scorer())])))
}

fn computer_player(
        initial_game_state: &GameState,
        moves: &Vec<Move>,
        name: String,
        eval_function: Box<Fn(&GameState) -> i16>)
        -> Move {

    if moves.len() == 0 {
        panic!("No possible moves passed to computer player!");
    }

    let move_scores = determine_best_moves(&initial_game_state, moves, &eval_function, MAX_DEPTH).0;
    if let [(ref best_move, _), ..] = move_scores.as_slice() {
        println!("Total moves possible: {}", move_scores.len());
        println!("Best moves according to the {} AI ({:?}):\n{}",
            name,
            initial_game_state.current_player,
            move_scores.clone().into_iter()
                 .take(5)
                 .fold("".to_owned(), |mut text, (piece_move, score)| {
                       text.push_str(format!("{}: {}, ", piece_move.simple_format(), score).as_str());
                       text
                 }));
        return best_move.clone();
    }

    panic!(format!("No moves returned by player {:?}", initial_game_state.current_player));
}

// Returns a list of pairs of moves with scores, sorted from best to worst.
fn determine_best_moves(
        initial_game_state: &GameState,
        moves: &Vec<Move>,
        eval_function: &Box<Fn(&GameState) -> i16>,
        ply: u8)
        -> (Vec<(Move, i16)>, i16) {

    if ply == 0 {
        panic!("Zero ply specified!");
    }

    match initial_game_state.get_end_state(&moves) {
        EndState::NotEnded => (),
        EndState::Win(_) => return (
            vec![],
            1000 * if initial_game_state.current_player == Color::White { -1 } else { 1 }),
        EndState::Stalemate => return (
            moves.iter().zip([0].iter().cycle()).map(|(s, c)| (s.clone(), c.clone())).collect::<Vec<_>>(),
            0),
    }

    let current_player = initial_game_state.current_player;

    let mut move_scores: Vec<(Move, i16)> = vec![];
    for piece_move in moves {
        let mut game_state = initial_game_state.clone();
        game_state.move_piece(&piece_move);

        let score = if ply > 1 {
            // Determine the other player's best move, returning 0 if there isn't a move,
            // indicating stalemate.
            let next_moves = game_state.get_player_moves_without_check(game_state.current_player);
            match determine_best_moves(&game_state, &next_moves, eval_function, ply - 1) {
                (_, score) => score,
            }
        } else {
            // Use the base, non-recursive heuristic if we are only looking ahead one move.
            eval_function(&game_state)
        };

        move_scores.push((piece_move.clone(), score));
    }

    move_scores.sort_by(|&(_, score0), &(_, score1)|
        if current_player == Color::White { score1.cmp(&score0) } else { score0.cmp(&score1) });

    let best_score = move_scores[0].1.clone();
    (move_scores, best_score)
}

fn multi_eval(
        game_state: &GameState,
        scorers: &[(i16, &Box<Fn(&GameState, &[Move], &[Move]) -> i16>)]) -> i16 {

    let mut score = 0;

    let white_moves = game_state.get_player_moves_without_check(Color::White);
    let black_moves = game_state.get_player_moves_without_check(Color::Black);

    for &(weight, scorer) in scorers {
        if weight != 0 {
            score += weight * scorer(&game_state, &white_moves, &black_moves);
        }
    }

    score
}

fn moves_scorer() -> Box<Fn(&GameState, &[Move], &[Move]) -> i16> {
    Box::new(|_, white_moves, black_moves| white_moves.len() as i16 - black_moves.len() as i16)
}

fn spaces_scorer() -> Box<Fn(&GameState, &[Move], &[Move]) -> i16> {
    Box::new(|_, white_moves, black_moves| {
        let mut ownership_grid = [[0; 8]; 8];

        for white_move in white_moves {
            let dest = white_move.destination.clone();
            ownership_grid[dest.row as usize][dest.column as usize] += 1;
        }

        for white_move in black_moves {
            let dest = white_move.destination.clone();
            ownership_grid[dest.row as usize][dest.column as usize] -= 1;
        }

        let mut space_score = 0;
        for col in 0..8 {
            for row in 0..8 {
                match ownership_grid[row][col].cmp(&0) {
                    Ordering::Less => space_score -= 1,
                    Ordering::Greater => space_score += 1,
                    _ => (),
                }
            }
        }

        space_score
    })
}

// TODO: Instead of iterating over the entire board to get all pieces, just use the move lists.
fn piece_scorer() -> Box<Fn(&GameState, &[Move], &[Move]) -> i16> {
    Box::new(|game_state, _, _| {
        game_state.get_all_pieces().iter().map(|piece| {
            let sign = if piece.color == Color::White { 1 } else { -1 };
            sign * piece_value(&piece.piece_type) as i16
        }).fold(0, |x, y| x + y)
    })
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
