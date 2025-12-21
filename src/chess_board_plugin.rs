use bevy::prelude::*;

pub const SCREEN_WIDTH: f32 = 800.0;
pub const SCREEN_HEIGHT: f32 = 800.0;
pub const TILE_SIZE: f32 = 800.0 / 8.0;
pub struct ChessBoardPlugin;

const DARK_TILE: Color = Color::srgb_u8(32, 32, 35);
const LIGHT_TILE: Color = Color::srgb_u8(235, 235, 235);

impl Plugin for ChessBoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_camera, setup_chessboard).chain());
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
            let color: Color = if (i + j) % 2 == 0 {
                DARK_TILE
            } else {
                LIGHT_TILE
            };

            commands.spawn((
                Mesh2d(rect_mesh.clone()),
                MeshMaterial2d(materials.add(color)),
                Transform::from_xyz(
                    i as f32 * TILE_SIZE - SCREEN_WIDTH / 2.0 + TILE_SIZE / 2.0,
                    j as f32 * TILE_SIZE - SCREEN_HEIGHT / 2.0 + TILE_SIZE / 2.0,
                    2.0,
                ),
            ));
        }
    }
}
