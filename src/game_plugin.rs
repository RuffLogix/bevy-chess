use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    chess_board_plugin::{SCREEN_HEIGHT, SCREEN_WIDTH, TILE_SIZE, is_valid_move},
    chess_piece_plugin::ChessPiece,
};

pub struct GamePlugin;

#[derive(Resource)]
pub struct GameState {
    pub chosen_chess_position: Option<(u32, u32)>,
    pub chosen_piece: Option<ChessPiece>,
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameState {
            chosen_chess_position: None,
            chosen_piece: None,
        });
        app.add_systems(Update, mouse_event);
    }
}

fn mouse_event(
    window: Single<&Window, With<PrimaryWindow>>,
    mut game_state: ResMut<GameState>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut chess_pieces: Query<(&mut ChessPiece, &mut Transform)>,
) {
    if let Some(position) = window.cursor_position() {
        let mouse_x_index: u32 = (position.x / TILE_SIZE) as u32;
        let mouse_y_index: u32 = ((SCREEN_HEIGHT - position.y) / TILE_SIZE) as u32;

        if buttons.just_pressed(MouseButton::Left) {
            let mut is_piece = false;
            for (chess_piece, _) in chess_pieces.iter() {
                if mouse_x_index == chess_piece.x_index && mouse_y_index == chess_piece.y_index {
                    is_piece = true;
                    if game_state.chosen_chess_position == None {
                        game_state.chosen_chess_position =
                            Some((chess_piece.x_index, chess_piece.y_index));
                        game_state.chosen_piece = Some(*chess_piece);
                    } else if let Some(game_position) = game_state.chosen_chess_position
                        && game_position.0 == chess_piece.x_index
                        && game_position.1 == chess_piece.y_index
                    {
                        game_state.chosen_chess_position = None;
                        game_state.chosen_piece = None;
                    }
                }
            }

            if !is_piece
                && let Some(chess_position) = game_state.chosen_chess_position
                && let Some(chess_piece) = game_state.chosen_piece
                && is_valid_move(
                    (mouse_x_index, mouse_y_index),
                    &chess_piece,
                    chess_pieces.iter().map(|(p, _)| &*p).collect(),
                )
            {
                for (mut chess_piece, mut transform) in chess_pieces.iter_mut() {
                    let i = chess_position.0;
                    let j = chess_position.1;

                    if i == chess_piece.x_index && j == chess_piece.y_index {
                        chess_piece.x_index = mouse_x_index;
                        chess_piece.y_index = mouse_y_index;
                        chess_piece.is_first_move = false;

                        transform.translation.x =
                            mouse_x_index as f32 * TILE_SIZE - SCREEN_WIDTH / 2.0 + TILE_SIZE / 2.0;
                        transform.translation.y =
                            mouse_y_index as f32 * TILE_SIZE - SCREEN_WIDTH / 2.0 + TILE_SIZE / 2.0;
                    }
                }

                game_state.chosen_chess_position = None;
                game_state.chosen_piece = None;
            }
        }
    }
}
