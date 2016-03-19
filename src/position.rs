#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub column: i8,
    pub row: i8,
}

impl Position {
    pub fn from_notation(notation: &str) -> Option<Position> {
        if notation.len() != 2 {
            return None;
        }

        let mut chars = notation.chars();
        let col = match chars.next().unwrap() {
                'a' => Some(0),
                'b' => Some(1),
                'c' => Some(2),
                'd' => Some(3),
                'e' => Some(4),
                'f' => Some(5),
                'g' => Some(6),
                'h' => Some(7),
                _   => None,
        };

        if col.is_none() {
            return None;
        } 
        if let Some(row) = chars.next().unwrap().to_digit(10) {
            if row == 0 || row > 8 {
                return None;
            }
            return Some(Position {
                column: col.unwrap(),
                row: (row - 1) as i8,
            })
        } else {
            return None;
        }
    }

    pub fn relative(&self, column_offset: i8, row_offset: i8) -> Position {
        Position { column: self.column + column_offset, row: self.row + row_offset }
    }

    pub fn format(&self) -> String {
        let column_text = match self.column {
            0 => "a",
            1 => "b",
            2 => "c",
            3 => "d",
            4 => "e",
            5 => "f",
            6 => "g",
            7 => "h",
            _ => panic!("column values must be between 0 and 7 inclusive")
        };

        let row_text = (self.row + 1).to_string();
        
        format!("{}{}", column_text, row_text)
    }
}
