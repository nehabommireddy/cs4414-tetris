use std::collections::{HashSet, VecDeque};

use crate::bot::{BoardState, CellState, TetrisAction, TetrominoType};

const WIDTH: usize = 10;
const HEIGHT: usize = 20;
const SEARCH_LIMIT: usize = 1000;

type Grid = [[bool; WIDTH]; HEIGHT];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Position {
    col: i32,
    row: i32,
    rotation: usize,
}

#[derive(Clone)]
struct SearchNode {
    position: Position,
    actions: Vec<TetrisAction>,
}

fn piece_cells(shape: TetrominoType, rotation: usize) -> [(i32, i32); 4] {
    let states = match shape {
        TetrominoType::I => [
            [[-1, 0], [0, 0], [1, 0], [2, 0]],
            [[0, 1], [0, 0], [0, -1], [0, -2]],
            [[-2, 0], [-1, 0], [0, 0], [1, 0]],
            [[0, 2], [0, 1], [0, 0], [0, -1]],
        ],
        TetrominoType::J => [
            [[-1, 1], [-1, 0], [0, 0], [1, 0]],
            [[0, 1], [1, 1], [0, 0], [0, -1]],
            [[-1, 0], [0, 0], [1, 0], [1, -1]],
            [[0, 1], [0, 0], [0, -1], [-1, -1]],
        ],
        TetrominoType::L => [
            [[-1, 0], [0, 0], [1, 0], [1, 1]],
            [[0, 1], [0, 0], [0, -1], [1, -1]],
            [[-1, 0], [0, 0], [1, 0], [-1, -1]],
            [[-1, 1], [0, 1], [0, 0], [0, -1]],
        ],
        TetrominoType::O => [
            [[0, 1], [1, 1], [0, 0], [1, 0]],
            [[0, 0], [1, 0], [0, -1], [1, -1]],
            [[-1, 0], [0, 0], [-1, -1], [0, -1]],
            [[-1, 1], [0, 1], [-1, 0], [0, 0]],
        ],
        TetrominoType::S => [
            [[0, 1], [1, 1], [-1, 0], [0, 0]],
            [[0, 1], [0, 0], [1, 0], [1, -1]],
            [[0, 0], [1, 0], [-1, -1], [0, -1]],
            [[-1, 1], [-1, 0], [0, 0], [0, -1]],
        ],
        TetrominoType::T => [
            [[0, 0], [0, 1], [-1, 0], [1, 0]],
            [[0, 0], [0, 1], [1, 0], [0, -1]],
            [[-1, 0], [0, 0], [0, -1], [1, 0]],
            [[-1, 0], [0, 0], [0, 1], [0, -1]],
        ],
        TetrominoType::Z => [
            [[-1, 1], [0, 1], [0, 0], [1, 0]],
            [[1, 1], [0, 0], [1, 0], [0, -1]],
            [[-1, 0], [0, 0], [0, -1], [1, -1]],
            [[0, 1], [-1, 0], [0, 0], [-1, -1]],
        ],
    };

    states[rotation % 4].map(|[x, y]| (x, y))
}

fn kick_offsets(shape: TetrominoType, rotation: usize) -> &'static [[i32; 2]] {
    const NORMAL: [[[i32; 2]; 5]; 4] = [
        [[0, 0], [0, 0], [0, 0], [0, 0], [0, 0]],
        [[0, 0], [1, 0], [1, -1], [0, 2], [1, 2]],
        [[0, 0], [0, 0], [0, 0], [0, 0], [0, 0]],
        [[0, 0], [-1, 0], [-1, -1], [0, 2], [-1, 2]],
    ];

    const I: [[[i32; 2]; 5]; 4] = [
        [[0, 0], [-1, 0], [2, 0], [-1, 0], [2, 0]],
        [[-1, 0], [0, 0], [0, 0], [0, 1], [0, -2]],
        [[-1, 1], [1, 1], [-2, 1], [1, 0], [-2, 0]],
        [[0, 1], [0, 1], [0, 1], [0, -1], [0, 2]],
    ];

    const O: [[[i32; 2]; 1]; 4] = [[[0, 0]], [[0, -1]], [[-1, -1]], [[-1, 0]]];

    match shape {
        TetrominoType::I => &I[rotation],
        TetrominoType::O => &O[rotation],
        _ => &NORMAL[rotation],
    }
}

fn copy_board(state: &BoardState) -> Grid {
    let mut board = [[false; WIDTH]; HEIGHT];

    for row in 0..HEIGHT {
        for col in 0..WIDTH {
            board[row][col] = state.board[row][col] == CellState::Filled;
        }
    }

    board
}

fn placeable(board: &Grid, shape: TetrominoType, position: Position) -> bool {
    for (dx, dy) in piece_cells(shape, position.rotation) {
        let col = position.col + dx;
        let row = position.row + dy;

        if col < 0 || col >= WIDTH as i32 || row < 0 {
            return false;
        }

        if row < HEIGHT as i32 && board[row as usize][col as usize] {
            return false;
        }
    }

    true
}

fn try_action(
    board: &Grid,
    shape: TetrominoType,
    position: Position,
    action: TetrisAction,
) -> Option<Position> {
    let mut next = position;

    match action {
        TetrisAction::L => next.col -= 1,
        TetrisAction::R => next.col += 1,
        TetrisAction::C | TetrisAction::W => {
            let target_rotation = if action == TetrisAction::C {
                (position.rotation + 1) % 4
            } else {
                (position.rotation + 3) % 4
            };

            let from = kick_offsets(shape, position.rotation);
            let to = kick_offsets(shape, target_rotation);

            for (from_offset, to_offset) in from.iter().zip(to.iter()) {
                let candidate = Position {
                    col: position.col + from_offset[0] - to_offset[0],
                    row: position.row + from_offset[1] - to_offset[1],
                    rotation: target_rotation,
                };

                if placeable(board, shape, candidate) {
                    return Some(candidate);
                }
            }

            return None;
        }
        _ => return None,
    }

    placeable(board, shape, next).then_some(next)
}

fn clear_lines(board: &mut Grid) -> usize {
    let mut next_row = 0;
    let mut cleared = 0;

    for row in 0..HEIGHT {
        if board[row].iter().all(|&cell| cell) {
            cleared += 1;
        } else {
            board[next_row] = board[row];
            next_row += 1;
        }
    }

    for row in next_row..HEIGHT {
        board[row] = [false; WIDTH];
    }

    cleared
}

fn simulate_drop(
    board: &Grid,
    shape: TetrominoType,
    mut position: Position,
) -> Option<(Grid, usize)> {
    if !placeable(board, shape, position) {
        return None;
    }

    while placeable(
        board,
        shape,
        Position {
            row: position.row - 1,
            ..position
        },
    ) {
        position.row -= 1;
    }

    let mut result = *board;

    for (dx, dy) in piece_cells(shape, position.rotation) {
        let col = position.col + dx;
        let row = position.row + dy;

        if row >= HEIGHT as i32 {
            return None;
        }

        result[row as usize][col as usize] = true;
    }

    let cleared = clear_lines(&mut result);
    Some((result, cleared))
}

fn evaluate_board(board: &Grid, cleared_lines: usize) -> f64 {
    let mut heights = [0i32; WIDTH];
    let mut holes = 0;
    let mut total_height = 0;

    for col in 0..WIDTH {
        let mut found_block = false;

        for row in (0..HEIGHT).rev() {
            if board[row][col] {
                if !found_block {
                    heights[col] = row as i32 + 1;
                    found_block = true;
                }
            } else if found_block {
                holes += 1;
            }
        }

        total_height += heights[col];
    }

    let bumpiness: i32 = heights
        .windows(2)
        .map(|pair| (pair[0] - pair[1]).abs())
        .sum();

    let maximum_height = *heights.iter().max().unwrap_or(&0);

    total_height as f64
    + 10.0 * holes as f64
    + bumpiness as f64
    + 2.0 * maximum_height as f64
    - 5.0 * cleared_lines as f64
}

pub fn find_best_actions(state: &BoardState) -> Vec<TetrisAction> {
    let board = copy_board(state);

    let start = Position {
        col: state.tetromino_col as i32,
        row: state.tetromino_row as i32,
        rotation: state.tetromino_rotation,
    };

    if !placeable(&board, state.tetromino, start) {
        return vec![TetrisAction::H];
    }

    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();

    queue.push_back(SearchNode {
        position: start,
        actions: Vec::new(),
    });
    visited.insert(start);

    let mut best_score = f64::INFINITY;
    let mut best_actions = vec![TetrisAction::H];

    while let Some(node) = queue.pop_front() {
        if let Some((result, cleared)) = simulate_drop(&board, state.tetromino, node.position) {
            let score = evaluate_board(&result, cleared) + node.actions.len() as f64 * 0.03;

            if score < best_score {
                best_score = score;
                best_actions = node.actions.clone();
                best_actions.push(TetrisAction::H);
            }
        }

        if visited.len() >= SEARCH_LIMIT {
            continue;
        }

        for action in [
            TetrisAction::L,
            TetrisAction::R,
            TetrisAction::C,
            TetrisAction::W,
        ] {
            let Some(next) = try_action(&board, state.tetromino, node.position, action) else {
                continue;
            };

            if visited.insert(next) {
                let mut actions = node.actions.clone();
                actions.push(action);

                queue.push_back(SearchNode {
                    position: next,
                    actions,
                });
            }
        }
    }

    best_actions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_board_has_zero_score() {
        let board = [[false; WIDTH]; HEIGHT];
        assert_eq!(evaluate_board(&board, 0), 0.0);
    }

    #[test]
    fn finds_actions_for_empty_board() {
        let state = BoardState {
            tetromino_row: 20,
            tetromino_col: 4,
            tetromino_rotation: 0,
            tetromino: TetrominoType::T,
            board: [[CellState::Empty; 10]; 20],
        };

        let actions = find_best_actions(&state);
        assert_eq!(actions.last(), Some(&TetrisAction::H));
    }
}
