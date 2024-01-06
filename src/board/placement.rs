use crate::board::piece::Piece;
use crate::board::square::Square;

use Rotation::*;

use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Rotation {
    Zero,
    Quarter,
    Half,
    ThreeQuarter,
}

impl Rotation {
    pub fn all_by_symmetry(symmetry: u8) -> Vec<Rotation> {
        match symmetry % 4 {
            0 => vec![Zero],
            2 => vec![Zero, Quarter],
            _ => vec![Zero, Quarter, Half, ThreeQuarter],
        }
    }
}

impl From<u8> for Rotation {
    fn from(amount: u8) -> Self {
        match amount % 4 {
            0 => Zero,
            1 => Quarter,
            2 => Half,
            3 => ThreeQuarter,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct Placement {
    pub piece: Piece,
    pub square: Square,
    pub rotation: Rotation,
    pub mirror: bool,
}

#[derive(Debug)]
pub struct PlacementError;

impl Error for PlacementError {}

impl fmt::Display for PlacementError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Placement does not produce valid board")
    }
}

