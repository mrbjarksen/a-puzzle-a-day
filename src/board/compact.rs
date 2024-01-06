use crate::board::Board;
use crate::board::piece::Piece;
use crate::board::square::Square;
use crate::board::path::Path;

use crate::board::placement::*;
use Rotation::*;

#[derive(Debug)]
pub struct CompactBoard {
    squares: [Option<Square>; 8],
    rotations: [Rotation; 8],
    mirrors: [bool; 8],
}

impl TryFrom<CompactBoard> for Board {
    type Error = PlacementError;

    fn try_from(cb: CompactBoard) -> Result<Self, Self::Error> {
        let mut board = Board::new();
        let pieces = Piece::pieces();
        for i in 0..8 {
            if let (Some(square), rotation, mirror, piece) = (cb.squares[i], cb.rotations[i], cb.mirrors[i], pieces[i]) {
                let path = Path::from_orientation(piece, rotation, mirror);
                board = board.place(piece, square, &path).ok_or(PlacementError)?;
            }
        }
        Ok(board)
    }
}

impl TryFrom<Board> for CompactBoard {
    type Error = PlacementError;

    fn try_from(board: Board) -> Result<Self, Self::Error> {
        let mut squares = [None; 8];
        let mut rotations = [Zero; 8];
        let mut mirrors = [false; 8];

        for Placement { piece, square, rotation, mirror } in board.placements()? {
            squares[piece as usize] = Some(square);
            rotations[piece as usize] = rotation;
            mirrors[piece as usize] = mirror;
        }

        Ok(CompactBoard { squares, rotations, mirrors })
    }
}

impl CompactBoard {
    pub fn to_bytes(&self) -> [u8; 9] {
        let mut bytes = [0; 9];

        for mirror in self.mirrors {
            bytes[0] = (bytes[0] << 1) | mirror as u8;
        }

        for (i, &rotation) in self.rotations.iter().enumerate() {
            bytes[i+1] = (rotation as u8) << 6
        }

        for (i, &square) in self.squares.iter().enumerate() {
            bytes[i+1] |= match square {
                None => 0b111111,
                Some(square) => square as u8,
            }
        }

        bytes
    }
}

impl From<[u8; 9]> for CompactBoard {
    fn from(bytes: [u8; 9]) -> Self {
        let mut squares = [None; 8];
        let mut rotations = [Zero; 8];
        let mut mirrors = [false; 8];

        for i in 0..8 {
            squares[i] = Square::try_from(bytes[i+1] & 0b111111).ok();
            rotations[i] = Rotation::from((bytes[i+1] >> 6) & 0b11);
            mirrors[i] = (bytes[0] >> (7 - i)) & 0b1 == 1;
        }

        CompactBoard { squares, rotations, mirrors }
    }
}
