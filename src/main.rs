mod chess_board_plugin;
mod chess_piece_plugin;

use bevy::prelude::*;
use chess_board_plugin::ChessBoardPlugin;
use chess_piece_plugin::ChessPiecePlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Chess Game in Bevy".into(),
                        resolution: (800, 800).into(),
                        resizable: false,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins((ChessBoardPlugin, ChessPiecePlugin))
        .run();
}
