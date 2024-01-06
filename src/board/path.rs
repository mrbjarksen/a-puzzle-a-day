use crate::board::piece::Piece;
use crate::board::placement::Rotation;

use Direction::*;

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Debug)]
pub struct Path(Vec<Direction>);

impl From<Vec<Direction>> for Path {
    fn from(directions: Vec<Direction>) -> Self {
        Path(directions)
    }
}

impl Path {
    pub fn directions(&self) -> &Vec<Direction> {
        &self.0
    }

    pub fn rotate(&self, amount: Rotation) -> Self {
        let mut path = self.clone();
        for _ in 0..amount as u8 {
            for dir in path.0.iter_mut() {
                *dir = match *dir {
                    Up    => Left,
                    Down  => Right,
                    Left  => Down,
                    Right => Up,
                }
            }
        }
        path
    }

    pub fn mirror(&self) -> Self {
        let directions = self.0.iter().map(|dir| match dir {
            Up    => Up,
            Down  => Down,
            Left  => Right,
            Right => Left,
        }).collect();
        Path(directions)
    }

    pub fn from_orientation(piece: Piece, rotation: Rotation, mirror: bool) -> Self {
        let mut path = Path::from(piece);
        if mirror { path = path.mirror() }
        path.rotate(rotation)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Direction> {
        self.0.iter()
    }
    
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Direction> {
        self.0.iter_mut()
    }
}
