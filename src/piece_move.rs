use position::Position;
use piece_type::PieceType;

#[derive(Debug, PartialEq, Clone)]
pub struct Move {
    pub source: Position,
    pub destination: Position,
    pub enables_en_passant: bool,
    pub en_passant_target: Option<Position>,
    pub extra_castling_move: Option<ExtraCastlingMove>,
    pub promotion_piece_type: Option<PieceType>
}

#[derive(Debug, PartialEq, Clone)]
pub struct ExtraCastlingMove {
    pub source: Position,
    pub destination: Position,
}

impl Move {
    pub fn simple(source: Position, destination: Position) -> Move {
        Move {
            source: source,
            destination: destination,
            enables_en_passant: false,
            en_passant_target: None,
            extra_castling_move: None,
            promotion_piece_type: None,
        }
    }

    pub fn en_passant(source: Position, destination: Position, target: Position) -> Move {
        Move {
            source: source,
            destination: destination,
            enables_en_passant: false,
            en_passant_target: Some(target),
            extra_castling_move: None,
            promotion_piece_type: None,
        }
    }

    pub fn en_passant_enabler(source: Position, destination: Position) -> Move {
        Move {
            source: source,
            destination: destination,
            enables_en_passant: true,
            en_passant_target: None,
            extra_castling_move: None,
            promotion_piece_type: None,
        }
    }

    // Ideally this would take a Move rather than an Option<Move>,
    // but I couldn't figure out how to get the lifetime correct on the inlined Some().
    // I was able to do this for the None cases by referencing a static constant in global scope.
    pub fn castle(
            source: Position,
            destination: Position,
            extra_castling_move: ExtraCastlingMove) -> Move {

        Move {
            source: source,
            destination: destination,
            enables_en_passant: false,
            en_passant_target: None,
            extra_castling_move: Some(extra_castling_move),
            promotion_piece_type: None,
        }
    }

    pub fn promotion(source: Position, destination: Position, promotion_piece_type: PieceType) -> Move {
        Move {
            source: source,
            destination: destination,
            enables_en_passant: false,
            en_passant_target: None,
            extra_castling_move: None,
            promotion_piece_type: Some(promotion_piece_type),
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

    pub fn simple_format(&self) -> String {
        format!("{}{}", self.source.format(), self.destination.format())
    }
}
