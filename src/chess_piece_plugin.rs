use bevy::prelude::*;

use crate::chess_board_plugin::{SCREEN_HEIGHT, SCREEN_WIDTH, TILE_SIZE};
use crate::components::{GridPosition, Piece, PieceColor, PieceType};

pub struct ChessPiecePlugin;

impl Plugin for ChessPiecePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_chesspieces);
    }
}

pub fn setup_chesspieces(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("pieces.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::new(10, 10), 7, 4, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    for i in 0..8 {
        commands.spawn(get_chess_entity(
            i,
            1,
            PieceType::Pawn,
            PieceColor::White,
            &texture,
            &texture_atlas_layout,
        ));
        commands.spawn(get_chess_entity(
            i,
            6,
            PieceType::Pawn,
            PieceColor::Black,
            &texture,
            &texture_atlas_layout,
        ));
    }

    let back_rank = [
        PieceType::Rook,
        PieceType::Knight,
        PieceType::Bishop,
        PieceType::Queen,
        PieceType::King,
        PieceType::Bishop,
        PieceType::Knight,
        PieceType::Rook,
    ];

    for (file, piece_type) in back_rank.iter().enumerate() {
        commands.spawn(get_chess_entity(
            file as u32,
            0,
            *piece_type,
            PieceColor::White,
            &texture,
            &texture_atlas_layout,
        ));

        commands.spawn(get_chess_entity(
            file as u32,
            7,
            *piece_type,
            PieceColor::Black,
            &texture,
            &texture_atlas_layout,
        ));
    }
}

fn get_chess_entity(
    x: u32,
    y: u32,
    piece_type: PieceType,
    piece_color: PieceColor,
    texture: &Handle<Image>,
    texture_atlas_layout: &Handle<TextureAtlasLayout>,
) -> (Sprite, Transform, Piece, GridPosition) {
    (
        Sprite {
            custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
            ..Sprite::from_atlas_image(
                texture.clone(),
                TextureAtlas {
                    layout: texture_atlas_layout.clone(),
                    index: piece_type + piece_color,
                },
            )
        },
        Transform::from_xyz(
            x as f32 * TILE_SIZE - SCREEN_WIDTH / 2.0 + TILE_SIZE / 2.0,
            y as f32 * TILE_SIZE - SCREEN_HEIGHT / 2.0 + TILE_SIZE / 2.0,
            3.0,
        ),
        Piece::new(piece_color, piece_type),
        GridPosition::new(x, y),
    )
}
