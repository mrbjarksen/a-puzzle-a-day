pub mod path;
pub mod square;
pub mod piece;
pub mod placement;
pub mod compact;

pub use self::path::*;
pub use self::square::*;
pub use self::piece::*;
pub use self::placement::*;

use Status::*;

use std::fmt;

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd, Debug)]
pub struct Board([Status; 45]);

#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Debug)]
enum Status {
    Empty,
    Nonexistent,
    Occupied(Piece),
}

impl Default for Board {
    fn default() -> Self {
        let mut status = [Empty; 45];
        status[6] = Nonexistent;
        status[13] = Nonexistent;
        Board(status)
    }
}

impl Board {
    pub fn place(&self, piece: Piece, start: Square, path: &Path) -> Option<Self> {
        let mut status = self.0;
        let mut square = start;

        match self.0[square as usize] {
            Empty => status[square as usize] = Occupied(piece),
            _ => return None,
        }

        for dir in path.iter() {
            square = square.step(*dir)?;
            match self.0[square as usize] {
                Empty => status[square as usize] = Occupied(piece),
                _ => return None,
            }
        }

        Some(Board(status))
    }

    pub fn solved_for(&self) -> Option<Date> {
        let mut status = self.0.iter();
        
        let month = status.position(|&sq| sq == Empty)? as u8;
        let day = 1 + month + status.position(|&sq| sq == Empty)? as u8;

        let date = Date {
            month: Square::try_from(month).ok()?,
            day: Square::try_from(day).ok()?
        };

        match status.find(|&&sq| sq == Empty) {
            None if date.is_valid() => Some(date),
            _ => None,
        }
    }

    fn check_placement(&self, piece: Piece, start: Square) -> Option<(Rotation, bool)> {
        if self.0[start as usize] != Occupied(piece) {
            return None;
        }

        for &mirror in if piece.mirror_symmetric() { [false].iter() } else { [false, true].iter() } {
            for rotation in Rotation::all_by_symmetry(piece.rotational_symmetry()) {
                let path = Path::from_orientation(piece, rotation, mirror);
                let mut square = start;
                'walk: {
                    for &dir in path.iter() {
                        match square.step(dir) {
                            Some(new_square) if self.0[new_square as usize] == Occupied(piece) => { square = new_square; }
                            _ => { break 'walk; }
                        };
                    }
                    return Some((rotation, mirror));
                }
            }
        }

        None
    } 

    pub fn placements(&self) -> Result<Vec<Placement>, PlacementError> {
        if self.0[6] != Nonexistent || self.0[13] != Nonexistent {
            return Err(PlacementError);
        }

        if self.0.iter().filter(|&&status| status == Nonexistent).count() > 2 {
            return Err(PlacementError);
        }

        let mut placed = [false; 8];
        for piece in Piece::pieces() {
            let num_squares = self.0.iter().filter(|&&status| status == Occupied(piece)).count();
            match num_squares {
                0 => {}
                n if n == piece.square_count() => { placed[piece as usize] = true; }
                _ => { return Err(PlacementError); }
            }
        }

        let mut placements = [None; 8];
        for square in Square::squares() {
            if let Occupied(piece) = self.0[square as usize] {
                if let (None, Some((rotation, mirror))) = (placements[piece as usize], self.check_placement(piece, square)) {
                    placements[piece as usize] = Some((square, rotation, mirror));
                }
            }
        }

        let mut result = Vec::new();
        for piece in Piece::pieces() {
            match (placed[piece as usize], placements[piece as usize]) {
                (true, Some((square, rotation, mirror))) => {
                    result.push(Placement { square, piece, rotation, mirror });
                }
                _ => { return Err(PlacementError); }
            }
        }

        Ok(result)
    }

    fn get_statuses_by_corner(&self, row: usize, col: usize) -> (Status, Status, Status, Status) {
        let (mut a, mut b, mut c, mut d) = (Nonexistent, Nonexistent, Nonexistent, Nonexistent);

        let square = col + 7*row;
        if row <= 7 && col <= 7 {
            if row > 0 && col > 0 && square < 53 { a = self.0[square - 8]; }
            if row > 0 && col < 7 && square < 52 { b = self.0[square - 7]; }
            if row < 7 && col > 0 && square < 46 { c = self.0[square - 1]; }
            if row < 7 && col < 7 && square < 45 { d = self.0[square];     }
        }

        (a, b, c, d)
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "╭───────────────────────────╮")?;

        for row in 0..=7 {
            write!(f, "│ ")?;
            for col in 0..=7 {
                write!(f, "{}", match self.get_statuses_by_corner(row, col) {
                    (Nonexistent, Nonexistent, Nonexistent, Nonexistent) => if row == 7 { "    " } else { "" },

                    (Nonexistent, Nonexistent, _, Nonexistent) => "┐ ",
                    (_, Nonexistent, Nonexistent, Nonexistent) => "┘ ",

                    (a, Nonexistent, b, Nonexistent) if a == b => "│ ",
                    (_, Nonexistent, _, Nonexistent) => "┤ ",

                    (a, b, c, d) if a == b && a == c && a == d => "    ",
                    (a, b, c, d) if a == b && c == d => "────",
                    (a, b, c, d) if a == c && b == d => "│   ",
                    
                    (a, b, c, _) if a == b && a == c => "┌───",
                    (a, b, _, d) if a == b && a == d => "┐   ",
                    (a, _, c, d) if a == c && a == d => "└───",
                    (_, b, c, d) if b == c && b == d => "┘   ",
                    
                    (a, b, _, _) if a == b => "┬───",
                    (a, _, c, _) if a == c => "├───",
                    (_, b, _, d) if b == d => "┤   ",
                    (_, _, c, d) if c == d => "┴───",
                    
                    _ => "┼───",
                })?;
            }
            writeln!(f, "│")?; 

            if row < 7 {
                write!(f, "│ ")?;
                for col in 0..=7 {
                    match self.get_statuses_by_corner(row, col) {
                        _ if row < 2 && col == 6 => (),
                        _ if row == 6 && col == 7 => write!(f, "  ")?,
                        _ if col == 7 => write!(f, "│ ")?,
                        (_, _, a, b) => {
                            if a == b { write!(f, " ")?; }
                            else { write!(f, "│")?; }
                            if b == Empty {
                                if let Ok(square) = Square::try_from((col + 7*row) as u8) {
                                    write!(f, "{:^3}", square)?;
                                }
                                else { write!(f, "   ")?; }
                            }
                            else { write!(f, "   ")?; }
                        }
                    };
                }
                if row == 1 { writeln!(f, "╰───╮")?; }
                else { writeln!(f, "│")?; }
            }
        }

        write!(f, "╰───────────────────────────────╯")?;

        Ok(())
    }
}

impl Board {
    pub fn to_mini_string(&self) -> String {
        let mut result = String::new();

        for row in 0..=7 {
            for col in 0..=7 {
                result.push_str(match self.get_statuses_by_corner(row, col) {
                    (Nonexistent, Nonexistent, Nonexistent, Nonexistent) => if row == 7 { "  " } else { "" },

                    (Nonexistent, Nonexistent, _, Nonexistent) => "┐\n",
                    (_, Nonexistent, Nonexistent, Nonexistent) => "┘\n",

                    (a, Nonexistent, b, Nonexistent) if a == b => "│\n",
                    (_, Nonexistent, _, Nonexistent) => "┤\n",

                    (a, b, c, d) if a == b && a == c && a == d => "  ",
                    (a, b, c, d) if a == b && c == d => "──",
                    (a, b, c, d) if a == c && b == d => "│ ",
                    
                    (a, b, c, _) if a == b && a == c => "┌─",
                    (a, b, _, d) if a == b && a == d => "┐ ",
                    (a, _, c, d) if a == c && a == d => "└─",
                    (_, b, c, d) if b == c && b == d => "┘ ",
                    
                    (a, b, _, _) if a == b => "┬─",
                    (a, _, c, _) if a == c => "├─",
                    (_, b, _, d) if b == d => "┤ ",
                    (_, _, c, d) if c == d => "┴─",
                    
                    _ => "┼─",
                });
            }
        }
        result
    }
}
