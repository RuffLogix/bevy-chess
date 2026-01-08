use crate::components::GridPosition;
use bevy::prelude::*;

#[derive(Message)]
pub struct PieceSelected;

#[derive(Message)]
pub struct PieceDeselected;

#[derive(Message)]
pub struct MoveMade {
    pub entity: Entity,
    pub from: GridPosition,
    pub to: GridPosition,
}

#[derive(Message)]
pub struct TileClicked {
    pub position: GridPosition,
}
