use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    chess_board_plugin::{BoardCell, SCREEN_HEIGHT, SCREEN_WIDTH, TILE_SIZE},
    components::{GridPosition, Piece, PieceColor, PieceType},
    events::{MoveMade, PieceDeselected, PieceSelected, TileClicked},
    resources::GameState,
    rules::{get_valid_moves, is_checkmate, is_self_check, is_square_under_attack},
};

pub struct GamePlugin;

#[derive(Component)]
struct StatusText;

#[derive(Component)]
struct MoveHistoryText;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameState>()
            .add_message::<TileClicked>()
            .add_message::<PieceSelected>()
            .add_message::<PieceDeselected>()
            .add_message::<MoveMade>()
            .add_systems(Startup, setup_ui)
            .add_systems(
                Update,
                (
                    input_system,
                    selection_logic_system,
                    move_execution_system,
                    highlight_moves_system,
                    update_ui_system,
                    check_game_status_system,
                )
                    .chain(),
            );
    }
}

fn setup_ui(mut commands: Commands) {
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

fn input_system(
    window: Single<&Window, With<PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut tile_clicked_events: MessageWriter<TileClicked>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        if let Some(position) = window.cursor_position() {
            let x = (position.x / TILE_SIZE).floor() as i32;
            let y = ((SCREEN_HEIGHT - position.y) / TILE_SIZE).floor() as i32;

            if (0..8).contains(&x) && (0..8).contains(&y) {
                tile_clicked_events.write(TileClicked {
                    position: GridPosition::new(x as u32, y as u32),
                });
            }
        }
    }
}

fn selection_logic_system(
    mut tile_clicked_events: MessageReader<TileClicked>,
    mut game_state: ResMut<GameState>,
    pieces: Query<(Entity, &GridPosition, &Piece)>,
    mut piece_selected_events: MessageWriter<PieceSelected>,
    mut piece_deselected_events: MessageWriter<PieceDeselected>,
    mut move_made_events: MessageWriter<MoveMade>,
) {
    for event in tile_clicked_events.read() {
        let clicked_pos = event.position;
        let mut clicked_piece_entity = None;

        for (entity, pos, piece) in pieces.iter() {
            if *pos == clicked_pos {
                clicked_piece_entity = Some((entity, piece));
                break;
            }
        }

        if let Some((entity, piece)) = clicked_piece_entity {
            if piece.color == game_state.current_turn {
                // Clicked own piece
                if game_state.selected_entity == Some(entity) {
                    game_state.selected_entity = None;
                    game_state.selected_position = None;
                    piece_deselected_events.write(PieceDeselected);
                } else {
                    game_state.selected_entity = Some(entity);
                    game_state.selected_position = Some(clicked_pos);
                    piece_selected_events.write(PieceSelected);
                }
            } else {
                // Clicked enemy piece (Potential Capture)
                if let Some(selected_entity) = game_state.selected_entity {
                    if let Ok((_, selected_pos, selected_piece)) = pieces.get(selected_entity) {
                        let all_pieces: Vec<(GridPosition, Piece)> =
                            pieces.iter().map(|(_, p, piece)| (*p, *piece)).collect();

                        let valid_moves =
                            get_valid_moves(*selected_pos, selected_piece, &all_pieces);

                        if valid_moves.contains(&clicked_pos)
                            && !is_self_check(
                                *selected_pos,
                                clicked_pos,
                                selected_piece,
                                &all_pieces,
                                game_state.current_turn,
                            )
                        {
                            move_made_events.write(MoveMade {
                                entity: selected_entity,
                                from: *selected_pos,
                                to: clicked_pos,
                            });
                        }
                    }
                }
            }
        } else {
            // Clicked empty tile
            if let Some(selected_entity) = game_state.selected_entity {
                if let Ok((_, selected_pos, selected_piece)) = pieces.get(selected_entity) {
                    let all_pieces: Vec<(GridPosition, Piece)> =
                        pieces.iter().map(|(_, p, piece)| (*p, *piece)).collect();

                    let valid_moves = get_valid_moves(*selected_pos, selected_piece, &all_pieces);

                    if valid_moves.contains(&clicked_pos)
                        && !is_self_check(
                            *selected_pos,
                            clicked_pos,
                            selected_piece,
                            &all_pieces,
                            game_state.current_turn,
                        )
                    {
                        move_made_events.write(MoveMade {
                            entity: selected_entity,
                            from: *selected_pos,
                            to: clicked_pos,
                        });
                    } else {
                        game_state.selected_entity = None;
                        game_state.selected_position = None;
                        piece_deselected_events.write(PieceDeselected);
                    }
                }
            }
        }
    }
}

fn move_execution_system(
    mut commands: Commands,
    mut move_events: MessageReader<MoveMade>,
    mut game_state: ResMut<GameState>,
    mut pieces: Query<(
        Entity,
        &mut GridPosition,
        &mut Piece,
        &mut Transform,
        &mut Sprite,
    )>,
) {
    for event in move_events.read() {
        let MoveMade { entity, from, to } = *event;

        let Ok((_, _, p, _, _)) = pieces.get(entity) else {
            continue;
        };
        let piece_kind = p.kind;
        let piece_color = p.color;

        let mut captured_entity = None;
        for (e, pos, _, _, _) in pieces.iter() {
            if *pos == to && e != entity {
                captured_entity = Some(e);
                break;
            }
        }

        let mut en_passant_victim = None;
        if piece_kind == PieceType::Pawn && from.x != to.x && captured_entity.is_none() {
            let victim_y = from.y;
            let victim_x = to.x;
            for (e, pos, _, _, _) in pieces.iter() {
                if pos.x == victim_x && pos.y == victim_y && e != entity {
                    en_passant_victim = Some(e);
                    break;
                }
            }
        }

        let mut castling_rook_info = None;
        if piece_kind == PieceType::King && (from.x as i32 - to.x as i32).abs() == 2 {
            let rank = from.y;
            let (rook_old_x, rook_new_x) = if to.x == 6 { (7, 5) } else { (0, 3) };
            for (e, pos, _, _, _) in pieces.iter() {
                if pos.x == rook_old_x && pos.y == rank {
                    castling_rook_info = Some((e, *pos, GridPosition::new(rook_new_x, rank)));
                    break;
                }
            }
        }

        // Mutations
        let move_str = record_move_string(piece_kind, piece_color, from, to);
        game_state.move_history.push(move_str);

        if let Some(e) = captured_entity {
            commands.entity(e).despawn();
        }
        if let Some(e) = en_passant_victim {
            commands.entity(e).despawn();
        }

        if let Some((rook_e, _, rook_new_pos)) = castling_rook_info {
            if let Ok((_, mut r_pos, _, mut r_transform, _)) = pieces.get_mut(rook_e) {
                *r_pos = rook_new_pos;
                update_transform(&mut r_transform, &rook_new_pos);
            }
        }

        let is_double_jump =
            piece_kind == PieceType::Pawn && (from.y as i32 - to.y as i32).abs() == 2;
        let is_promotion = piece_kind == PieceType::Pawn
            && ((piece_color == PieceColor::White && to.y == 7)
                || (piece_color == PieceColor::Black && to.y == 0));

        if let Ok((_, mut pos, mut piece, mut transform, mut sprite)) = pieces.get_mut(entity) {
            *pos = to;
            piece.is_first_move = false;
            piece.just_double_jumped = is_double_jump;

            if is_promotion {
                piece.kind = PieceType::Queen;
                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.index = PieceType::Queen + piece.color;
                }
            }

            update_transform(&mut transform, &to);
        }

        for (e, _, mut p, _, _) in pieces.iter_mut() {
            if e != entity {
                p.just_double_jumped = false;
            }
        }

        game_state.current_turn = match game_state.current_turn {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
        };
        game_state.selected_entity = None;
        game_state.selected_position = None;
    }
}

fn update_transform(transform: &mut Transform, pos: &GridPosition) {
    transform.translation.x = pos.x as f32 * TILE_SIZE - SCREEN_WIDTH / 2.0 + TILE_SIZE / 2.0;
    transform.translation.y = pos.y as f32 * TILE_SIZE - SCREEN_HEIGHT / 2.0 + TILE_SIZE / 2.0;
}

fn record_move_string(
    kind: PieceType,
    color: PieceColor,
    from: GridPosition,
    to: GridPosition,
) -> String {
    let from_str = get_chess_notation(from);
    let to_str = get_chess_notation(to);
    let piece_str = match kind {
        PieceType::Pawn => "P",
        PieceType::Knight => "N",
        PieceType::Bishop => "B",
        PieceType::Rook => "R",
        PieceType::Queen => "Q",
        PieceType::King => "K",
    };

    let color_prefix = match color {
        PieceColor::White => "W",
        PieceColor::Black => "B",
    };

    format!("{color_prefix}: {piece_str} {from_str} -> {to_str}")
}

fn get_chess_notation(pos: GridPosition) -> String {
    let file = (b'a' + pos.x as u8) as char;
    let rank = pos.y + 1;
    format!("{file}{rank}")
}

fn check_game_status_system(
    mut game_state: ResMut<GameState>,
    pieces: Query<(&GridPosition, &Piece)>,
    move_events: MessageReader<MoveMade>,
) {
    if !move_events.is_empty() {
        let all_pieces: Vec<(GridPosition, Piece)> =
            pieces.iter().map(|(pos, piece)| (*pos, *piece)).collect();

        if is_checkmate(game_state.current_turn, &all_pieces) {
            let king_pos = all_pieces
                .iter()
                .find(|(_, p)| p.kind == PieceType::King && p.color == game_state.current_turn)
                .map(|(pos, _)| *pos);

            let mut is_check = false;
            if let Some(k_pos) = king_pos {
                let opponent = match game_state.current_turn {
                    PieceColor::White => PieceColor::Black,
                    PieceColor::Black => PieceColor::White,
                };
                if is_square_under_attack(k_pos, &all_pieces, opponent) {
                    is_check = true;
                }
            }

            if is_check {
                let winner = match game_state.current_turn {
                    PieceColor::White => PieceColor::Black,
                    PieceColor::Black => PieceColor::White,
                };
                game_state.status = format!("Checkmate! {winner:?} wins.");
            } else {
                game_state.status = "Stalemate!".to_string();
            }
        } else {
            game_state.status = format!("{:?}'s Turn", game_state.current_turn);
        }
    }
}

fn highlight_moves_system(
    game_state: Res<GameState>,
    pieces: Query<(&GridPosition, &Piece)>,
    mut cells: Query<(&BoardCell, &Transform, &mut Sprite)>,
) {
    const DARK_TILE: Color = Color::srgb_u8(32, 32, 35);
    const DARK_RED_TILE: Color = Color::srgb_u8(100, 16, 16);
    const LIGHT_TILE: Color = Color::srgb_u8(235, 235, 235);
    const LIGHT_RED_TILE: Color = Color::srgb_u8(235, 120, 120);

    let mut valid_moves = Vec::new();

    if let Some(selected_entity) = game_state.selected_entity {
        if let Ok((selected_pos, selected_piece)) = pieces.get(selected_entity) {
            let all_pieces: Vec<(GridPosition, Piece)> =
                pieces.iter().map(|(pos, piece)| (*pos, *piece)).collect();
            valid_moves = get_valid_moves(*selected_pos, selected_piece, &all_pieces);
            valid_moves.retain(|to| {
                !is_self_check(
                    *selected_pos,
                    *to,
                    selected_piece,
                    &all_pieces,
                    game_state.current_turn,
                )
            });
            valid_moves.push(*selected_pos);
        }
    }

    for (_cell, transform, mut sprite) in cells.iter_mut() {
        let x = ((transform.translation.x - TILE_SIZE / 2.0 + SCREEN_WIDTH / 2.0) / TILE_SIZE)
            .round() as u32;
        let y = ((transform.translation.y - TILE_SIZE / 2.0 + SCREEN_HEIGHT / 2.0) / TILE_SIZE)
            .round() as u32;
        let pos = GridPosition::new(x, y);

        let is_highlighted = valid_moves.contains(&pos);

        if (pos.x + pos.y) % 2 == 0 {
            sprite.color = if is_highlighted {
                DARK_RED_TILE
            } else {
                DARK_TILE
            };
        } else {
            sprite.color = if is_highlighted {
                LIGHT_RED_TILE
            } else {
                LIGHT_TILE
            };
        }
    }
}

fn update_ui_system(
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
