use std::ops::Add;

use bevy::prelude::*;

use crate::chess_board_plugin::{SCREEN_HEIGHT, SCREEN_WIDTH, TILE_SIZE};

pub struct ChessPiecePlugin;

pub const BISHOP_MOVES: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
pub const ROOK_MOVES: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
pub const KING_MOVES: [(i32, i32); 8] = [
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
];
pub const KNIGHT_MOVES: [(i32, i32); 8] = [
    (2, 1),
    (1, 2),
    (-1, 2),
    (-2, 1),
    (-2, -1),
    (-1, -2),
    (1, -2),
    (2, -1),
];
pub const QUEEN_MOVES: [(i32, i32); 8] = [
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
];

#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChessPieceType {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChessPieceColor {
    White = 7,  // White pieces start at index 7 in the atlas
    Black = 14, // Black pieces start at index 14 in the atlas
}

#[derive(Component, Debug, PartialEq, Eq, Clone, Copy)]
pub struct ChessPiece {
    pub x_index: u32,
    pub y_index: u32,
    pub chess_color: ChessPieceColor,
    pub chess_type: ChessPieceType,
    pub is_first_move: bool,
}

impl Add<ChessPieceColor> for ChessPieceType {
    type Output = usize;

    fn add(self, rhs: ChessPieceColor) -> Self::Output {
        self as usize + rhs as usize
    }
}

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
            ChessPieceType::Pawn,
            ChessPieceColor::White,
            &texture,
            &texture_atlas_layout,
        ));
        commands.spawn(get_chess_entity(
            i,
            6,
            ChessPieceType::Pawn,
            ChessPieceColor::Black,
            &texture,
            &texture_atlas_layout,
        ));
    }

    let back_rank = [
        ChessPieceType::Rook,
        ChessPieceType::Knight,
        ChessPieceType::Bishop,
        ChessPieceType::Queen,
        ChessPieceType::King,
        ChessPieceType::Bishop,
        ChessPieceType::Knight,
        ChessPieceType::Rook,
    ];

    for (file, piece) in back_rank.iter().enumerate() {
        commands.spawn(get_chess_entity(
            file as u32,
            0,
            *piece,
            ChessPieceColor::White,
            &texture,
            &texture_atlas_layout,
        ));

        commands.spawn(get_chess_entity(
            file as u32,
            7,
            *piece,
            ChessPieceColor::Black,
            &texture,
            &texture_atlas_layout,
        ));
    }
}

fn get_chess_entity(
    i: u32,
    j: u32,
    chess_piece_type: ChessPieceType,
    chess_piece_color: ChessPieceColor,
    texture: &Handle<Image>,
    texture_atlas_layout: &Handle<TextureAtlasLayout>,
) -> (Sprite, Transform, ChessPiece) {
    (
        Sprite {
            custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
            ..Sprite::from_atlas_image(
                texture.clone(),
                TextureAtlas {
                    layout: texture_atlas_layout.clone(),
                    index: chess_piece_type + chess_piece_color,
                },
            )
        },
        Transform::from_xyz(
            i as f32 * TILE_SIZE - SCREEN_WIDTH / 2.0 + TILE_SIZE / 2.0,
            j as f32 * TILE_SIZE - SCREEN_HEIGHT / 2.0 + TILE_SIZE / 2.0,
            3.0,
        ),
        ChessPiece {
            x_index: i,
            y_index: j,
            chess_color: chess_piece_color,
            chess_type: chess_piece_type,
            is_first_move: true,
        },
    )
}
