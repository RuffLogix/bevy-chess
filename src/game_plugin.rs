use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    chess_board_plugin::{SCREEN_HEIGHT, SCREEN_WIDTH, TILE_SIZE, is_valid_move},
    chess_piece_plugin::{
        ChessPiece, ChessPieceColor, ChessPieceType, is_checkmate, is_self_check,
    },
};

pub struct GamePlugin;

#[derive(Resource)]
pub struct GameState {
    pub chosen_chess_position: Option<(u32, u32)>,
    pub chosen_piece: Option<ChessPiece>,
    pub current_turn: ChessPieceColor,
    pub status: String,
    pub move_history: Vec<String>,
}

#[derive(Component)]
struct StatusText;

#[derive(Component)]
struct MoveHistoryText;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameState {
            chosen_chess_position: None,
            chosen_piece: None,
            current_turn: ChessPieceColor::White,
            status: "White's Turn".to_string(),
            move_history: Vec::new(),
        });
        app.add_systems(Startup, setup_ui);
        app.add_systems(Update, (mouse_event, update_ui));
    }
}

fn setup_ui(mut commands: Commands) {
    // Container for Status
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(820.0),
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Status: White's Turn"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                StatusText,
            ));
        });

    // Container for Move History
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(50.0),
                left: Val::Px(820.0),
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Move History:\n"),
                TextFont {
                    font_size: 15.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                MoveHistoryText,
            ));
        });
}

fn update_ui(
    game_state: Res<GameState>,
    mut status_query: Query<&mut Text, (With<StatusText>, Without<MoveHistoryText>)>,
    mut history_query: Query<&mut Text, (With<MoveHistoryText>, Without<StatusText>)>,
) {
    if game_state.is_changed() {
        for mut text in status_query.iter_mut() {
            text.0 = format!("Status: {}", game_state.status);
        }

        for mut text in history_query.iter_mut() {
            let start_index = if game_state.move_history.len() > 20 {
                game_state.move_history.len() - 20
            } else {
                0
            };

            let history_str = game_state.move_history[start_index..]
                .iter()
                .fold(String::from("Move History:\n"), |acc, m| acc + m + "\n");
            text.0 = history_str;
        }
    }
}

fn mouse_event(
    window: Single<&Window, With<PrimaryWindow>>,
    mut game_state: ResMut<GameState>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut chess_pieces: Query<(Entity, &mut ChessPiece, &mut Transform, &mut Sprite)>,
) {
    if let Some(position) = window.cursor_position() {
        let mouse_x_index: u32 = (position.x / TILE_SIZE) as u32;
        let mouse_y_index: u32 = ((SCREEN_HEIGHT - position.y) / TILE_SIZE) as u32;

        if mouse_x_index < 8 && mouse_y_index < 8 && buttons.just_pressed(MouseButton::Left) {
            let mut is_piece = false;
            let mut removed_entity: Option<Entity> = None;
            for (entity, chess_piece, _, _) in chess_pieces.iter() {
                if mouse_x_index == chess_piece.x_index && mouse_y_index == chess_piece.y_index {
                    is_piece = true;
                    if game_state.chosen_chess_position.is_none()
                        && game_state.current_turn == chess_piece.chess_color
                    {
                        // Select piece
                        game_state.chosen_chess_position =
                            Some((chess_piece.x_index, chess_piece.y_index));
                        game_state.chosen_piece = Some(*chess_piece);
                    } else if let Some(game_position) = game_state.chosen_chess_position
                        && game_position.0 == chess_piece.x_index
                        && game_position.1 == chess_piece.y_index
                    {
                        // Deselect piece
                        game_state.chosen_chess_position = None;
                        game_state.chosen_piece = None;
                    } else if let Some(chess_piece_selected) = game_state.chosen_piece
                        && chess_piece_selected.chess_color != chess_piece.chess_color
                        && is_valid_move(
                            (mouse_x_index, mouse_y_index),
                            &chess_piece_selected,
                            chess_pieces.iter().map(|(_, p, _, _)| p).collect(),
                        )
                        && !is_self_check(
                            (mouse_x_index, mouse_y_index),
                            &chess_piece_selected,
                            chess_pieces.iter().map(|(_, p, _, _)| p).collect(),
                            game_state.current_turn,
                        )
                    {
                        // Capture piece
                        removed_entity = Some(entity);
                    }
                }
            }

            if let Some(entity) = removed_entity {
                commands.entity(entity).despawn();

                if let Some(chess_piece_selected) = game_state.chosen_piece {
                    record_move(
                        &mut game_state,
                        &chess_piece_selected,
                        (mouse_x_index, mouse_y_index),
                    );

                    for (_entity, mut chess_piece, mut transform, mut sprite) in
                        chess_pieces.iter_mut()
                    {
                        chess_piece.just_double_jumped = false;

                        if chess_piece.x_index == chess_piece_selected.x_index
                            && chess_piece.y_index == chess_piece_selected.y_index
                        {
                            update_piece_position(
                                &mut chess_piece,
                                &mut transform,
                                &mut sprite,
                                mouse_x_index,
                                mouse_y_index,
                            );
                        }
                    }

                    end_turn(&mut game_state);
                    check_game_status(&mut game_state, &chess_pieces);
                }
            }

            if !is_piece
                && let Some(chess_position) = game_state.chosen_chess_position
                && let Some(chess_piece) = game_state.chosen_piece
                && is_valid_move(
                    (mouse_x_index, mouse_y_index),
                    &chess_piece,
                    chess_pieces.iter().map(|(_, p, _, _)| p).collect(),
                )
                && !is_self_check(
                    (mouse_x_index, mouse_y_index),
                    &chess_piece,
                    chess_pieces.iter().map(|(_, p, _, _)| p).collect(),
                    game_state.current_turn,
                )
            {
                record_move(
                    &mut game_state,
                    &chess_piece,
                    (mouse_x_index, mouse_y_index),
                );

                let mut castling_move: Option<((u32, u32), (u32, u32))> = None;

                if chess_piece.chess_type == ChessPieceType::Pawn
                    && mouse_x_index != chess_position.0
                {
                    let captured_pos = (mouse_x_index, chess_position.1);
                    for (entity, piece, _, _) in chess_pieces.iter() {
                        if piece.x_index == captured_pos.0 && piece.y_index == captured_pos.1 {
                            commands.entity(entity).despawn();
                            break;
                        }
                    }
                }

                for (_entity, mut chess_piece, mut transform, mut sprite) in chess_pieces.iter_mut()
                {
                    chess_piece.just_double_jumped = false;

                    let i = chess_position.0;
                    let j = chess_position.1;

                    if i == chess_piece.x_index && j == chess_piece.y_index {
                        if chess_piece.chess_type == ChessPieceType::King
                            && (mouse_x_index as i32 - i as i32).abs() == 2
                        {
                            let (rook_old_x, rook_new_x) =
                                if mouse_x_index == 6 { (7, 5) } else { (0, 3) };
                            castling_move = Some(((rook_old_x, j), (rook_new_x, j)));
                        }

                        update_piece_position(
                            &mut chess_piece,
                            &mut transform,
                            &mut sprite,
                            mouse_x_index,
                            mouse_y_index,
                        );
                    }
                }

                if let Some(((old_x, old_y), (new_x, new_y))) = castling_move {
                    for (_entity, mut chess_piece, mut transform, mut sprite) in
                        chess_pieces.iter_mut()
                    {
                        if chess_piece.x_index == old_x && chess_piece.y_index == old_y {
                            update_piece_position(
                                &mut chess_piece,
                                &mut transform,
                                &mut sprite,
                                new_x,
                                new_y,
                            );
                        }
                    }
                }

                end_turn(&mut game_state);
                check_game_status(&mut game_state, &chess_pieces);
            }
        }
    }
}

fn check_game_status(
    game_state: &mut GameState,
    chess_pieces: &Query<(Entity, &mut ChessPiece, &mut Transform, &mut Sprite)>,
) {
    let pieces_vec: Vec<&ChessPiece> = chess_pieces.iter().map(|(_, p, _, _)| &*p).collect();

    if is_checkmate(game_state.current_turn, pieces_vec.clone()) {
        if let Some(piece) = pieces_vec
            .iter()
            .find(|p| p.chess_color == game_state.current_turn)
        {
            if is_self_check(
                (piece.x_index, piece.y_index),
                piece,
                pieces_vec,
                game_state.current_turn,
            ) {
                let winner = match game_state.current_turn {
                    ChessPieceColor::White => ChessPieceColor::Black,
                    ChessPieceColor::Black => ChessPieceColor::White,
                };
                game_state.status = format!("Checkmate! {:?} wins.", winner);
                println!("{}", game_state.status);
            } else {
                game_state.status = "Stalemate!".to_string();
                println!("{}", game_state.status);
            }
        }
    } else {
        game_state.status = format!("{:?}'s Turn", game_state.current_turn);
    }
}

fn update_piece_position(
    chess_piece: &mut ChessPiece,
    transform: &mut Transform,
    sprite: &mut Sprite,
    mouse_x_index: u32,
    mouse_y_index: u32,
) {
    if chess_piece.chess_type == ChessPieceType::Pawn {
        if (chess_piece.y_index as i32 - mouse_y_index as i32).abs() == 2 {
            chess_piece.just_double_jumped = true;
        }

        if (chess_piece.chess_color == ChessPieceColor::White && mouse_y_index == 7)
            || (chess_piece.chess_color == ChessPieceColor::Black && mouse_y_index == 0)
        {
            chess_piece.chess_type = ChessPieceType::Queen;
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = chess_piece.chess_type + chess_piece.chess_color;
            }
        }
    }

    chess_piece.x_index = mouse_x_index;
    chess_piece.y_index = mouse_y_index;
    chess_piece.is_first_move = false;

    transform.translation.x =
        mouse_x_index as f32 * TILE_SIZE - SCREEN_WIDTH / 2.0 + TILE_SIZE / 2.0;
    transform.translation.y =
        mouse_y_index as f32 * TILE_SIZE - SCREEN_HEIGHT / 2.0 + TILE_SIZE / 2.0;
}

fn end_turn(game_state: &mut ResMut<GameState>) {
    if game_state.current_turn == ChessPieceColor::White {
        game_state.current_turn = ChessPieceColor::Black;
    } else {
        game_state.current_turn = ChessPieceColor::White;
    }
    game_state.chosen_chess_position = None;
    game_state.chosen_piece = None;
}

fn record_move(game_state: &mut GameState, piece: &ChessPiece, to: (u32, u32)) {
    let from_str = get_chess_notation(piece.x_index, piece.y_index);
    let to_str = get_chess_notation(to.0, to.1);
    let piece_str = match piece.chess_type {
        ChessPieceType::Pawn => "P",
        ChessPieceType::Knight => "N",
        ChessPieceType::Bishop => "B",
        ChessPieceType::Rook => "R",
        ChessPieceType::Queen => "Q",
        ChessPieceType::King => "K",
    };

    let color_prefix = match piece.chess_color {
        ChessPieceColor::White => "W",
        ChessPieceColor::Black => "B",
    };

    let move_str = format!("{}: {} {} -> {}", color_prefix, piece_str, from_str, to_str);

    game_state.move_history.push(move_str);
}

fn get_chess_notation(x: u32, y: u32) -> String {
    let file = (b'a' + x as u8) as char;
    let rank = y + 1;
    format!("{}{}", file, rank)
}
