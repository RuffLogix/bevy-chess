use bevy::prelude::*;

use crate::{
    chess_piece_plugin::{
        BISHOP_MOVES, ChessPiece, ChessPieceColor, ChessPieceType, KING_MOVES, KNIGHT_MOVES,
        QUEEN_MOVES, ROOK_MOVES,
    },
    game_plugin::GameState,
};

pub const SCREEN_WIDTH: f32 = 1200.0;
pub const SCREEN_HEIGHT: f32 = 800.0;
pub const TILE_SIZE: f32 = 800.0 / 8.0;
pub struct ChessBoardPlugin;

const DARK_TILE: Color = Color::srgb_u8(32, 32, 35);
const DARK_RED_TILE: Color = Color::srgb_u8(100, 16, 16);
const LIGHT_TILE: Color = Color::srgb_u8(235, 235, 235);
const LIGHT_RED_TILE: Color = Color::srgb_u8(235, 120, 120);

#[derive(Component)]
struct BoardCell(u32, u32);

impl Plugin for ChessBoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_camera, setup_chessboard).chain());
        app.add_systems(Update, draw_valid_move);
    }
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub fn setup_chessboard(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let rect_mesh = meshes.add(Rectangle::new(TILE_SIZE, TILE_SIZE));

    for i in 0..8 {
        for j in 0..8 {
            let color = get_cell_color(i, j, false);

            commands.spawn((
                Mesh2d(rect_mesh.clone()),
                MeshMaterial2d(materials.add(color)),
                Transform::from_xyz(
                    i as f32 * TILE_SIZE - SCREEN_WIDTH / 2.0 + TILE_SIZE / 2.0,
                    j as f32 * TILE_SIZE - SCREEN_HEIGHT / 2.0 + TILE_SIZE / 2.0,
                    2.0,
                ),
                BoardCell(i, j),
            ));
        }
    }
}

fn draw_valid_move(
    mut cells: Query<(&BoardCell, &mut MeshMaterial2d<ColorMaterial>)>,
    chess_pieces: Query<&ChessPiece>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    game_state: Res<GameState>,
) {
    let mut valid_cells: Vec<(u32, u32)> = Vec::new();
    for chess_piece in chess_pieces.iter() {
        if let Some(chose_position) = game_state.chosen_chess_position {
            if chose_position.0 == chess_piece.x_index && chose_position.1 == chess_piece.y_index {
                valid_cells = get_valid_moves(chess_piece, chess_pieces.iter().collect());
                break;
            }
        }
    }

    for (cell, material) in cells.iter_mut() {
        let (i, j) = (cell.0, cell.1);
        let mut is_valid_move = false;
        for (ii, jj) in valid_cells.iter() {
            if i == *ii && j == *jj {
                is_valid_move = true;
                break;
            }
        }

        let color = get_cell_color(i, j, is_valid_move);

        if let Some(mat) = materials.get_mut(&material.0) {
            mat.color = color;
        }
    }
}

fn get_valid_moves(chess_piece: &ChessPiece, chess_pieces: Vec<&ChessPiece>) -> Vec<(u32, u32)> {
    let mut res: Vec<(u32, u32)> = Vec::from([(chess_piece.x_index, chess_piece.y_index)]);

    match chess_piece.chess_type {
        ChessPieceType::Pawn => check_pawn_move(&mut res, chess_piece, chess_pieces),
        ChessPieceType::Bishop => check_bishop_move(&mut res, chess_piece, chess_pieces),
        ChessPieceType::King => check_king_move(&mut res, chess_piece, chess_pieces),
        ChessPieceType::Knight => check_knight_move(&mut res, chess_piece, chess_pieces),
        ChessPieceType::Queen => check_queen_move(&mut res, chess_piece, chess_pieces),
        ChessPieceType::Rook => check_rook_move(&mut res, chess_piece, chess_pieces),
    }
    res
}

pub fn is_valid_move(
    target_position: (u32, u32),
    chess_piece: &ChessPiece,
    chess_pieces: Vec<&ChessPiece>,
) -> bool {
    let valid_moves = get_valid_moves(chess_piece, chess_pieces);
    for (i, j) in valid_moves.iter() {
        if *i == target_position.0 && *j == target_position.1 {
            return true;
        }
    }
    false
}

fn get_cell_color(i: u32, j: u32, is_valid_move: bool) -> Color {
    if (i + j) % 2 == 0 {
        match is_valid_move {
            true => DARK_RED_TILE,
            false => DARK_TILE,
        }
    } else {
        match is_valid_move {
            true => LIGHT_RED_TILE,
            false => LIGHT_TILE,
        }
    }
}

fn check_pawn_move(
    res: &mut Vec<(u32, u32)>,
    chess_piece: &ChessPiece,
    chess_pieces: Vec<&ChessPiece>,
) {
    let direction: i32 = match chess_piece.chess_color {
        ChessPieceColor::White => 1,
        ChessPieceColor::Black => -1,
    };

    let one_step_y = chess_piece.y_index as i32 + direction;
    if (0..8).contains(&one_step_y) {
        let mut is_occupied = false;
        for other_piece in chess_pieces.iter() {
            if other_piece.x_index == chess_piece.x_index
                && other_piece.y_index == one_step_y as u32
            {
                is_occupied = true;
                break;
            }
        }
        if !is_occupied {
            res.push((chess_piece.x_index, one_step_y as u32));

            if chess_piece.is_first_move {
                let two_step_y = chess_piece.y_index as i32 + 2 * direction;
                if (0..8).contains(&two_step_y) {
                    let mut is_occupied = false;
                    for other_piece in chess_pieces.iter() {
                        if other_piece.x_index == chess_piece.x_index
                            && other_piece.y_index == two_step_y as u32
                        {
                            is_occupied = true;
                            break;
                        }
                    }
                    if !is_occupied {
                        res.push((chess_piece.x_index, two_step_y as u32));
                    }
                }
            }
        }

        for dx in [-1, 1].iter() {
            let new_x = chess_piece.x_index as i32 + dx;
            if !(0..8).contains(&new_x) {
                continue;
            }
            for other_piece in chess_pieces.iter() {
                if other_piece.x_index == new_x as u32
                    && other_piece.y_index == one_step_y as u32
                    && other_piece.chess_color != chess_piece.chess_color
                {
                    res.push((new_x as u32, one_step_y as u32));
                }
            }
        }

        // En Passant
        for dx in [-1, 1].iter() {
            let adj_x = chess_piece.x_index as i32 + dx;
            if !(0..8).contains(&adj_x) {
                continue;
            }

            for other_piece in chess_pieces.iter() {
                if other_piece.x_index == adj_x as u32
                    && other_piece.y_index == chess_piece.y_index
                    && other_piece.chess_color != chess_piece.chess_color
                    && other_piece.chess_type == ChessPieceType::Pawn
                    && other_piece.just_double_jumped
                {
                    res.push((adj_x as u32, one_step_y as u32));
                }
            }
        }
    }
}

fn check_bishop_move(
    res: &mut Vec<(u32, u32)>,
    chess_piece: &ChessPiece,
    chess_pieces: Vec<&ChessPiece>,
) {
    for (dx, dy) in BISHOP_MOVES.iter() {
        let mut step = 1;
        let mut first_opposing_piece_encountered = false;
        loop {
            let new_x = chess_piece.x_index as i32 + dx * step;
            let new_y = chess_piece.y_index as i32 + dy * step;
            if !(0..8).contains(&new_x) || !(0..8).contains(&new_y) {
                break;
            }

            let mut is_occupied = false;
            for other_piece in chess_pieces.iter() {
                if other_piece.x_index == new_x as u32 && other_piece.y_index == new_y as u32 {
                    if other_piece.chess_color != chess_piece.chess_color
                        && !first_opposing_piece_encountered
                    {
                        first_opposing_piece_encountered = true;
                        res.push((new_x as u32, new_y as u32));
                    }
                    is_occupied = true;
                    break;
                }
            }
            if is_occupied {
                break;
            } else {
                res.push((new_x as u32, new_y as u32));
            }
            step += 1;
        }
    }
}

fn check_king_move(
    res: &mut Vec<(u32, u32)>,
    chess_piece: &ChessPiece,
    chess_pieces: Vec<&ChessPiece>,
) {
    // Normal King Moves
    for (dx, dy) in KING_MOVES.iter() {
        let new_x = chess_piece.x_index as i32 + dx;
        let new_y = chess_piece.y_index as i32 + dy;
        if !(0..8).contains(&new_x) || !(0..8).contains(&new_y) {
            continue;
        }

        let mut is_occupied = false;
        for other_piece in chess_pieces.iter() {
            if other_piece.x_index == new_x as u32 && other_piece.y_index == new_y as u32 {
                if other_piece.chess_color != chess_piece.chess_color {
                    res.push((new_x as u32, new_y as u32));
                }
                is_occupied = true;
                break;
            }
        }
        if !is_occupied {
            res.push((new_x as u32, new_y as u32));
        }
    }

    // Castling Logic
    if chess_piece.is_first_move {
        let (rank, opponent_color) = match chess_piece.chess_color {
            ChessPieceColor::White => (0, ChessPieceColor::Black),
            ChessPieceColor::Black => (7, ChessPieceColor::White),
        };

        // Cannot castle if King is currently in check
        if is_square_under_attack(
            (chess_piece.x_index, chess_piece.y_index),
            &chess_pieces,
            opponent_color,
        ) {
            return;
        }

        // Kingside Castling
        // King moves to (6, rank). Rook at (7, rank). Empty (5, rank), (6, rank).
        // Transit square (5, rank) and destination (6, rank) must not be attacked.
        if check_castling_path(
            chess_piece,
            &chess_pieces,
            rank,
            opponent_color,
            7,       // rook x
            &[5, 6], // empty squares
            &[5, 6], // safe squares
        ) {
            res.push((6, rank));
        }

        // Queenside Castling
        // King moves to (2, rank). Rook at (0, rank). Empty (1, rank), (2, rank), (3, rank).
        // Transit square (3, rank) and destination (2, rank) must not be attacked.
        if check_castling_path(
            chess_piece,
            &chess_pieces,
            rank,
            opponent_color,
            0,          // rook x
            &[1, 2, 3], // empty squares
            &[2, 3],    // safe squares
        ) {
            res.push((2, rank));
        }
    }
}

fn check_castling_path(
    king: &ChessPiece,
    all_pieces: &Vec<&ChessPiece>,
    rank: u32,
    opponent_color: ChessPieceColor,
    rook_x: u32,
    empty_xs: &[u32],
    safe_xs: &[u32],
) -> bool {
    // 1. Check Rook existence and first move
    let rook_exists = all_pieces.iter().any(|p| {
        p.x_index == rook_x
            && p.y_index == rank
            && p.chess_type == ChessPieceType::Rook
            && p.chess_color == king.chess_color
            && p.is_first_move
    });

    if !rook_exists {
        return false;
    }

    // 2. Check empty squares
    for &x in empty_xs {
        if all_pieces
            .iter()
            .any(|p| p.x_index == x && p.y_index == rank)
        {
            return false;
        }
    }

    // 3. Check safe squares (not attacked)
    for &x in safe_xs {
        if is_square_under_attack((x, rank), all_pieces, opponent_color) {
            return false;
        }
    }

    true
}

fn is_square_under_attack(
    position: (u32, u32),
    chess_pieces: &Vec<&ChessPiece>,
    opponent_color: ChessPieceColor,
) -> bool {
    for piece in chess_pieces {
        if piece.chess_color == opponent_color {
            match piece.chess_type {
                ChessPieceType::Pawn => {
                    let direction = match piece.chess_color {
                        ChessPieceColor::White => 1,
                        ChessPieceColor::Black => -1,
                    };
                    let attack_y = piece.y_index as i32 + direction;
                    if attack_y == position.1 as i32 {
                        if (piece.x_index as i32 - position.0 as i32).abs() == 1 {
                            return true;
                        }
                    }
                }
                ChessPieceType::King => {
                    let dx = (piece.x_index as i32 - position.0 as i32).abs();
                    let dy = (piece.y_index as i32 - position.1 as i32).abs();
                    if dx <= 1 && dy <= 1 {
                        return true;
                    }
                }
                _ => {
                    // For other pieces, use standard move validation
                    // We must avoid recursion by not calling is_valid_move for King type (handled above)
                    if is_valid_move(position, piece, chess_pieces.clone()) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn check_knight_move(
    res: &mut Vec<(u32, u32)>,
    chess_piece: &ChessPiece,
    chess_pieces: Vec<&ChessPiece>,
) {
    for (dx, dy) in KNIGHT_MOVES.iter() {
        let new_x = chess_piece.x_index as i32 + dx;
        let new_y = chess_piece.y_index as i32 + dy;
        if !(0..8).contains(&new_x) || !(0..8).contains(&new_y) {
            continue;
        }

        let mut is_occupied = false;
        for other_piece in chess_pieces.iter() {
            if other_piece.x_index == new_x as u32 && other_piece.y_index == new_y as u32 {
                if other_piece.chess_color != chess_piece.chess_color {
                    res.push((new_x as u32, new_y as u32));
                }
                is_occupied = true;
                break;
            }
        }
        if !is_occupied {
            res.push((new_x as u32, new_y as u32));
        }
    }
}

fn check_queen_move(
    res: &mut Vec<(u32, u32)>,
    chess_piece: &ChessPiece,
    chess_pieces: Vec<&ChessPiece>,
) {
    for (dx, dy) in QUEEN_MOVES.iter() {
        let mut step = 1;
        let mut first_opposing_piece_encountered = false;
        loop {
            let new_x = chess_piece.x_index as i32 + dx * step;
            let new_y = chess_piece.y_index as i32 + dy * step;
            if !(0..8).contains(&new_x) || !(0..8).contains(&new_y) {
                break;
            }

            let mut is_occupied = false;
            for other_piece in chess_pieces.iter() {
                if other_piece.x_index == new_x as u32 && other_piece.y_index == new_y as u32 {
                    if other_piece.chess_color != chess_piece.chess_color
                        && !first_opposing_piece_encountered
                    {
                        first_opposing_piece_encountered = true;
                        res.push((new_x as u32, new_y as u32));
                    }
                    is_occupied = true;
                    break;
                }
            }
            if is_occupied {
                break;
            } else {
                res.push((new_x as u32, new_y as u32));
            }
            step += 1;
        }
    }
}

fn check_rook_move(
    res: &mut Vec<(u32, u32)>,
    chess_piece: &ChessPiece,
    chess_pieces: Vec<&ChessPiece>,
) {
    for (dx, dy) in ROOK_MOVES.iter() {
        let mut step = 1;
        let mut first_opposing_piece_encountered = false;
        loop {
            let new_x = chess_piece.x_index as i32 + dx * step;
            let new_y = chess_piece.y_index as i32 + dy * step;
            if !(0..8).contains(&new_x) || !(0..8).contains(&new_y) {
                break;
            }

            let mut is_occupied = false;
            for other_piece in chess_pieces.iter() {
                if other_piece.x_index == new_x as u32 && other_piece.y_index == new_y as u32 {
                    if other_piece.chess_color != chess_piece.chess_color
                        && !first_opposing_piece_encountered
                    {
                        first_opposing_piece_encountered = true;
                        res.push((new_x as u32, new_y as u32));
                    }
                    is_occupied = true;
                    break;
                }
            }
            if is_occupied {
                break;
            } else {
                res.push((new_x as u32, new_y as u32));
            }
            step += 1;
        }
    }
}
