use position::Position;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Move {
    pub source: Position,
    pub destination: Position,
    pub enables_en_passant: bool,
    pub en_passant_target: Option<Position>,
}

impl Move {
    pub fn simple(source: Position, destination: Position) -> Move {
        Move {
            source: source,
            destination: destination,
            enables_en_passant: false,
            en_passant_target: None,
        }
    }

    pub fn en_passant(source: Position, destination: Position, target: Position) -> Move {
        Move {
            source: source,
            destination: destination,
            enables_en_passant: false,
            en_passant_target: Some(target),
        }
    }

    pub fn en_passant_enabler(source: Position, destination: Position) -> Move {
        Move {
            source: source,
            destination: destination,
            enables_en_passant: true,
            en_passant_target: None,
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

