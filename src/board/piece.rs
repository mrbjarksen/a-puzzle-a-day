use crate::board::path::{Direction::*, Path};

use std::fmt;

use Piece::*;

#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Debug)]
#[repr(u8)]
pub enum Piece {
    L, N, O, P, U, V, Y, Z,
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-piece", match self {
            L => 'L', N => 'N', O => 'O', P => 'P',
            U => 'U', V => 'V', Y => 'Y', Z => 'Z',
        })
    }
}

impl Piece {
    pub fn pieces() -> [Piece; 8] {
        [L, N, O, P, U, V, Y, Z]
    }

    pub fn rotational_symmetry(&self) -> u8 {
        match self {
            O | Z => 2,
            _ => 1,
        }
    }

    pub fn mirror_symmetric(&self) -> bool {
        match self {
            O | U | V => true,
            _ => false,
        }
    }

    pub fn square_count(&self) -> usize {
        match self {
            O => 6,
            _ => 5,
        }
    }
}

impl From<Piece> for Path {
    fn from(piece: Piece) -> Self {
        let directions = match piece {
            L => vec![Down, Down, Down, Right],
            N => vec![Up, Right, Up, Up],
            O => vec![Up, Up, Right, Down, Down],
            P => vec![Up, Up, Right, Down],
            U => vec![Down, Right, Right, Up],
            V => vec![Right, Right, Up, Up],
            Y => vec![Up, Up, Up, Down, Left],
            Z => vec![Right, Down, Down, Right],
        };
        Path::from(directions)
    }
}
