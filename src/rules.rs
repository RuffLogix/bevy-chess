use crate::components::{GridPosition, Piece, PieceColor, PieceType};

pub const BISHOP_OFFSETS: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
pub const ROOK_OFFSETS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
pub const KNIGHT_OFFSETS: [(i32, i32); 8] = [
    (2, 1),
    (1, 2),
    (-1, 2),
    (-2, 1),
    (-2, -1),
    (-1, -2),
    (1, -2),
    (2, -1),
];
pub const KING_OFFSETS: [(i32, i32); 8] = [
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
];
pub const QUEEN_OFFSETS: [(i32, i32); 8] = [
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
];

fn to_grid_pos(x: i32, y: i32) -> Option<GridPosition> {
    if (0..8).contains(&x) && (0..8).contains(&y) {
        Some(GridPosition::new(x as u32, y as u32))
    } else {
        None
    }
}

fn get_piece_at(pieces: &[(GridPosition, Piece)], x: u32, y: u32) -> Option<&Piece> {
    pieces
        .iter()
        .find(|(pos, _)| pos.x == x && pos.y == y)
        .map(|(_, p)| p)
}

pub fn get_valid_moves(
    position: GridPosition,
    piece: &Piece,
    pieces: &[(GridPosition, Piece)],
) -> Vec<GridPosition> {
    let mut moves = Vec::new();

    match piece.kind {
        PieceType::Pawn => get_pawn_moves(&mut moves, position, piece, pieces),
        PieceType::Knight => get_knight_moves(&mut moves, position, piece, pieces),
        PieceType::Bishop => {
            get_sliding_moves(&mut moves, position, piece, pieces, &BISHOP_OFFSETS)
        }
        PieceType::Rook => get_sliding_moves(&mut moves, position, piece, pieces, &ROOK_OFFSETS),
        PieceType::Queen => get_sliding_moves(&mut moves, position, piece, pieces, &QUEEN_OFFSETS),
        PieceType::King => get_king_moves(&mut moves, position, piece, pieces),
    }

    moves
}

fn get_pawn_moves(
    moves: &mut Vec<GridPosition>,
    position: GridPosition,
    piece: &Piece,
    pieces: &[(GridPosition, Piece)],
) {
    let direction: i32 = match piece.color {
        PieceColor::White => 1,
        PieceColor::Black => -1,
    };

    let start_y = position.y as i32;
    let start_x = position.x as i32;

    // 1. Single step forward
    let one_step_y = start_y + direction;
    if let Some(pos) = to_grid_pos(start_x, one_step_y) {
        if get_piece_at(pieces, pos.x, pos.y).is_none() {
            moves.push(pos);

            // 2. Double step forward
            if piece.is_first_move {
                let two_step_y = start_y + 2 * direction;
                if let Some(pos2) = to_grid_pos(start_x, two_step_y) {
                    if get_piece_at(pieces, pos2.x, pos2.y).is_none() {
                        moves.push(pos2);
                    }
                }
            }
        }
    }

    // 3. Captures
    for dx in [-1, 1] {
        let target_x = start_x + dx;
        let target_y = start_y + direction;

        if let Some(pos) = to_grid_pos(target_x, target_y) {
            // Normal capture
            if let Some(other_piece) = get_piece_at(pieces, pos.x, pos.y) {
                if other_piece.color != piece.color {
                    moves.push(pos);
                }
            } else {
                // En Passant
                // Check if there is an opponent pawn at (target_x, start_y) that just double jumped
                if let Some(other_pos) = to_grid_pos(target_x, start_y) {
                    if let Some(other_piece) = get_piece_at(pieces, other_pos.x, other_pos.y) {
                        if other_piece.kind == PieceType::Pawn
                            && other_piece.color != piece.color
                            && other_piece.just_double_jumped
                        {
                            moves.push(pos);
                        }
                    }
                }
            }
        }
    }
}

fn get_knight_moves(
    moves: &mut Vec<GridPosition>,
    position: GridPosition,
    piece: &Piece,
    pieces: &[(GridPosition, Piece)],
) {
    for (dx, dy) in KNIGHT_OFFSETS {
        let new_x = position.x as i32 + dx;
        let new_y = position.y as i32 + dy;

        if let Some(pos) = to_grid_pos(new_x, new_y) {
            if let Some(other_piece) = get_piece_at(pieces, pos.x, pos.y) {
                if other_piece.color != piece.color {
                    moves.push(pos);
                }
            } else {
                moves.push(pos);
            }
        }
    }
}

fn get_sliding_moves(
    moves: &mut Vec<GridPosition>,
    position: GridPosition,
    piece: &Piece,
    pieces: &[(GridPosition, Piece)],
    offsets: &[(i32, i32)],
) {
    for (dx, dy) in offsets {
        let mut step = 1;
        loop {
            let new_x = position.x as i32 + dx * step;
            let new_y = position.y as i32 + dy * step;

            if let Some(pos) = to_grid_pos(new_x, new_y) {
                if let Some(other_piece) = get_piece_at(pieces, pos.x, pos.y) {
                    if other_piece.color != piece.color {
                        moves.push(pos);
                    }
                    // Blocked by a piece (either friend or foe, we stop)
                    break;
                } else {
                    moves.push(pos);
                }
            } else {
                // Off board
                break;
            }
            step += 1;
        }
    }
}

fn get_king_moves(
    moves: &mut Vec<GridPosition>,
    position: GridPosition,
    piece: &Piece,
    pieces: &[(GridPosition, Piece)],
) {
    // Normal moves
    for (dx, dy) in KING_OFFSETS {
        let new_x = position.x as i32 + dx;
        let new_y = position.y as i32 + dy;

        if let Some(pos) = to_grid_pos(new_x, new_y) {
            if let Some(other_piece) = get_piece_at(pieces, pos.x, pos.y) {
                if other_piece.color != piece.color {
                    moves.push(pos);
                }
            } else {
                moves.push(pos);
            }
        }
    }

    // Castling
    if piece.is_first_move {
        let (rank, opponent_color) = match piece.color {
            PieceColor::White => (0, PieceColor::Black),
            PieceColor::Black => (7, PieceColor::White),
        };

        // Cannot castle if King is currently in check
        if is_square_under_attack(position, pieces, opponent_color) {
            return;
        }

        // Kingside (rook at x=7)
        if check_castling_path(
            piece,
            pieces,
            rank,
            opponent_color,
            7,
            &[5, 6], // empty squares between king(4) and rook(7)
            &[5, 6], // squares king passes through/lands on must be safe
        ) {
            moves.push(GridPosition::new(6, rank));
        }

        // Queenside (rook at x=0)
        if check_castling_path(
            piece,
            pieces,
            rank,
            opponent_color,
            0,
            &[1, 2, 3], // empty squares between king(4) and rook(0)
            &[2, 3],    // squares king passes through/lands on (4->3->2)
        ) {
            moves.push(GridPosition::new(2, rank));
        }
    }
}

fn check_castling_path(
    king: &Piece,
    pieces: &[(GridPosition, Piece)],
    rank: u32,
    opponent_color: PieceColor,
    rook_x: u32,
    empty_xs: &[u32],
    safe_xs: &[u32],
) -> bool {
    // 1. Check Rook existence and first move
    let rook_exists = pieces.iter().any(|(pos, p)| {
        pos.x == rook_x
            && pos.y == rank
            && p.kind == PieceType::Rook
            && p.color == king.color
            && p.is_first_move
    });

    if !rook_exists {
        return false;
    }

    // 2. Check empty squares
    for &x in empty_xs {
        if get_piece_at(pieces, x, rank).is_some() {
            return false;
        }
    }

    // 3. Check safe squares
    for &x in safe_xs {
        if is_square_under_attack(GridPosition::new(x, rank), pieces, opponent_color) {
            return false;
        }
    }

    true
}

pub fn is_square_under_attack(
    position: GridPosition,
    pieces: &[(GridPosition, Piece)],
    attacker_color: PieceColor,
) -> bool {
    for (pos, piece) in pieces {
        if piece.color == attacker_color {
            match piece.kind {
                PieceType::Pawn => {
                    let direction = match piece.color {
                        PieceColor::White => 1,
                        PieceColor::Black => -1,
                    };
                    // Check if pawn attacks 'position'
                    // Pawn at (pos.x, pos.y) attacks (pos.x +/- 1, pos.y + direction)
                    let attack_y = pos.y as i32 + direction;
                    if attack_y == position.y as i32
                        && (pos.x as i32 - position.x as i32).abs() == 1
                    {
                        return true;
                    }
                }
                PieceType::King => {
                    let dx = (pos.x as i32 - position.x as i32).abs();
                    let dy = (pos.y as i32 - position.y as i32).abs();
                    if dx <= 1 && dy <= 1 {
                        return true;
                    }
                }
                PieceType::Knight | PieceType::Bishop | PieceType::Rook | PieceType::Queen => {
                    // For these pieces, if 'position' is in their pseudo-legal moves (assuming 'position' contains a piece or is empty)
                    // We can reuse get_valid_moves, but we need to be careful.
                    // Actually, simplest is to see if they can move to 'position'.
                    // Note: get_valid_moves handles "capture" logic if there is an opponent piece.
                    // But here 'position' might be an empty square we are checking for safety.
                    // We need to pretend there is a piece at 'position' of the victim's color to see if it would be captured?
                    // Or simply check if the move is generated.

                    // Optimization: We don't want to generate ALL moves for every piece.
                    // But for now, let's reuse get_valid_moves but we must avoid infinite recursion if get_valid_moves calls is_square_under_attack (which it does for King castling).
                    // Luckily, we are inside the match arms for Knight/Bishop/Rook/Queen, so we won't recurse back to King castling logic.

                    let moves = get_valid_moves(*pos, piece, pieces);
                    if moves.contains(&position) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

pub fn is_self_check(
    from: GridPosition,
    to: GridPosition,
    piece: &Piece,
    pieces: &[(GridPosition, Piece)],
    current_turn: PieceColor,
) -> bool {
    // Simulate move
    // 1. Filter out captured piece at 'to'
    // 2. Update moved piece
    // 3. Handle En Passant removal

    let mut simulated_pieces: Vec<(GridPosition, Piece)> = pieces.to_vec();

    // Remove piece at destination if any (capture)
    simulated_pieces.retain(|(p, _)| *p != to);

    // En Passant capture special case
    if piece.kind == PieceType::Pawn && from.x != to.x && get_piece_at(pieces, to.x, to.y).is_none()
    {
        // This is an en passant move, remove the captured pawn
        // Captured pawn is at (to.x, from.y)
        simulated_pieces.retain(|(p, _)| !(p.x == to.x && p.y == from.y));
    }

    // Move the piece
    if let Some(idx) = simulated_pieces.iter().position(|(p, _)| *p == from) {
        simulated_pieces[idx].0 = to;
        // Note: we don't strictly need to update piece internal state like `is_first_move` for check detection
        // unless it affects subsequent logic. For simple check detection, geometry is key.
    }

    // Find King
    let king_pos = simulated_pieces.iter().find_map(|(pos, p)| {
        if p.kind == PieceType::King && p.color == current_turn {
            Some(*pos)
        } else {
            None
        }
    });

    if let Some(k_pos) = king_pos {
        let opponent_color = match current_turn {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
        };
        return is_square_under_attack(k_pos, &simulated_pieces, opponent_color);
    }

    false
}

pub fn is_checkmate(current_turn: PieceColor, pieces: &[(GridPosition, Piece)]) -> bool {
    // If any piece of current_turn has any legal move, it's not checkmate
    for (pos, piece) in pieces {
        if piece.color == current_turn {
            let moves = get_valid_moves(*pos, piece, pieces);
            for target in moves {
                if !is_self_check(*pos, target, piece, pieces, current_turn) {
                    return false;
                }
            }
        }
    }
    true
}
