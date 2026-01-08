use bevy::prelude::*;
use crate::components::{GridPosition, PieceColor};

#[derive(Resource)]
pub struct GameState {
    pub selected_entity: Option<Entity>,
    pub selected_position: Option<GridPosition>,
    pub current_turn: PieceColor,
    pub status: String,
    pub move_history: Vec<String>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            selected_entity: None,
            selected_position: None,
            current_turn: PieceColor::White,
            status: "White's Turn".to_string(),
            move_history: Vec::new(),
        }
    }
}
