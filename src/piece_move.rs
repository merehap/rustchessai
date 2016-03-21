use position::Position;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Move<'a> {
    pub source: Position,
    pub destination: Position,
    pub enables_en_passant: bool,
    pub en_passant_target: Option<Position>,
    pub extra_castling_move: &'a Option<Move<'a>>,
}

static NONE: Option<Move<'static>> = None;

impl<'a> Move<'a> {
    pub fn simple(source: Position, destination: Position) -> Move<'a> {
        Move {
            source: source,
            destination: destination,
            enables_en_passant: false,
            en_passant_target: None,
            extra_castling_move: &NONE,
        }
    }

    pub fn en_passant(source: Position, destination: Position, target: Position) -> Move<'a> {
        Move {
            source: source,
            destination: destination,
            enables_en_passant: false,
            en_passant_target: Some(target),
            extra_castling_move: &NONE,
        }
    }

    pub fn en_passant_enabler(source: Position, destination: Position) -> Move<'a> {
        Move {
            source: source,
            destination: destination,
            enables_en_passant: true,
            en_passant_target: None,
            extra_castling_move: &NONE,
        }
    }

    // Ideally this would take a Move rather than an Option<Move>,
    // but I couldn't figure out how to get the lifetime correct on the inlined Some().
    // I was able to do this for the None cases by referencing a static constant in global scope.
    pub fn castle(source: Position, destination: Position, extra_castling_move: &'a Option<Move<'a>>) -> Move<'a> {
        Move {
            source: source,
            destination: destination,
            enables_en_passant: false,
            en_passant_target: None,
            extra_castling_move: extra_castling_move,
        }
    }

    pub fn from_notation(notation: &str) -> Option<Move> {
        if notation.len() != 4 {
            return None;
        }

        let source = Position::from_notation(&notation[0..2].to_owned());
        let dest = Position::from_notation(&notation[2..4].to_owned());
        if source.is_none() || dest.is_none() {
            return None;
        }

        Some (Move::simple(source.unwrap(), dest.unwrap()))
    }
}
