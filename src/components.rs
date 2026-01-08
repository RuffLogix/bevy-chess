use bevy::prelude::*;
use std::ops::Add;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPosition {
    pub x: u32,
    pub y: u32,
}

impl GridPosition {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PieceType {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PieceColor {
    White = 7,
    Black = 14,
}

impl Add<PieceColor> for PieceType {
    type Output = usize;

    fn add(self, rhs: PieceColor) -> Self::Output {
        self as usize + rhs as usize
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub color: PieceColor,
    pub kind: PieceType,
    pub is_first_move: bool,
    pub just_double_jumped: bool,
}

impl Piece {
    pub fn new(color: PieceColor, kind: PieceType) -> Self {
        Self {
            color,
            kind,
            is_first_move: true,
            just_double_jumped: false,
        }
    }
}
