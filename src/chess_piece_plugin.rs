use std::ops::Add;

use bevy::prelude::*;

use crate::chess_board_plugin::{SCREEN_HEIGHT, SCREEN_WIDTH, TILE_SIZE, is_valid_move};

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
    pub just_double_jumped: bool,
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
            just_double_jumped: false,
        },
    )
}

// Check if the move is a self-check
// If the move would put the player's own king in check, return true
pub fn is_self_check(
    new_position: (u32, u32),
    chess_piece: &ChessPiece,
    all_pieces: Vec<&ChessPiece>,
    current_turn: ChessPieceColor,
) -> bool {
    let mut simulated_pieces: Vec<ChessPiece> = all_pieces.iter().map(|p| **p).collect();

    let is_en_passant = chess_piece.chess_type == ChessPieceType::Pawn
        && chess_piece.x_index != new_position.0
        && !all_pieces
            .iter()
            .any(|p| p.x_index == new_position.0 && p.y_index == new_position.1);

    for piece in simulated_pieces.iter_mut() {
        if piece.x_index == chess_piece.x_index && piece.y_index == chess_piece.y_index {
            piece.x_index = new_position.0;
            piece.y_index = new_position.1;
        }
    }
    simulated_pieces.retain(|p| {
        if is_en_passant
            && p.x_index == new_position.0
            && p.y_index == chess_piece.y_index
            && p.chess_color != chess_piece.chess_color
        {
            return false;
        }

        !(p.x_index == new_position.0
            && p.y_index == new_position.1
            && p.chess_color != chess_piece.chess_color)
    });

    let king_position = simulated_pieces.iter().find_map(|p| {
        if p.chess_type == ChessPieceType::King && p.chess_color == current_turn {
            Some((p.x_index, p.y_index))
        } else {
            None
        }
    });

    if king_position.is_none() {
        return false;
    }
    let king_position = king_position.unwrap();

    for piece in simulated_pieces.iter() {
        if piece.chess_color != current_turn
            && is_valid_move(king_position, piece, simulated_pieces.iter().collect())
        {
            return true;
        }
    }

    false
}

pub fn is_checkmate(current_turn: ChessPieceColor, all_pieces: Vec<&ChessPiece>) -> bool {
    for piece in all_pieces.iter().filter(|p| p.chess_color == current_turn) {
        for x in 0..8 {
            for y in 0..8 {
                let new_position = (x, y);
                if is_valid_move(new_position, piece, all_pieces.to_vec())
                    && !is_self_check(new_position, piece, all_pieces.to_vec(), current_turn)
                {
                    return false;
                }
            }
        }
    }
    true
}
