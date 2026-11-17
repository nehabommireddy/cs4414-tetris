use std::collections::VecDeque;

use crate::bot_strategy::find_best_actions;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TetrominoType {
    I,
    J,
    L,
    O,
    S,
    Z,
    T,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TetrisAction {
    C,
    W,
    L,
    R,
    D,
    H,
    E,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellState {
    Filled,
    Empty,
}

pub struct BoardState {
    pub tetromino_row: usize,
    pub tetromino_col: usize,
    pub tetromino_rotation: usize,
    pub tetromino: TetrominoType,
    pub board: [[CellState; 10]; 20],
}

pub trait SinglePlayerBot {
    fn new() -> Self;
    fn get_move(&mut self, board_state: &BoardState) -> TetrisAction;
}

// Supports the assignment's example importing Bot.
pub use SinglePlayerBot as Bot;

pub struct GarbageLine {
    pub num_rows_to_create: usize,
    pub ticks_till_applied: usize,
}

pub trait MultiPlayerBot {
    fn new() -> Self;

    fn get_move(
        &mut self,
        p1_state: &BoardState,
        p1_garbage: &[GarbageLine],
        p2_state: &BoardState,
        p2_garbage: &[GarbageLine],
    ) -> TetrisAction;
}

pub struct BadBot;

impl SinglePlayerBot for BadBot {
    fn new() -> Self {
        Self
    }

    fn get_move(&mut self, _state: &BoardState) -> TetrisAction {
        TetrisAction::H
    }
}

impl MultiPlayerBot for BadBot {
    fn new() -> Self {
        Self
    }

    fn get_move(
        &mut self,
        _p1_state: &BoardState,
        _p1_garbage: &[GarbageLine],
        _p2_state: &BoardState,
        _p2_garbage: &[GarbageLine],
    ) -> TetrisAction {
        TetrisAction::H
    }
}

pub struct GoodBot {
    planned_actions: VecDeque<TetrisAction>,
    previous_position: Option<(usize, usize, usize)>,
    previous_shape: Option<TetrominoType>,
    previous_action: Option<TetrisAction>,
}

impl GoodBot {
    fn create() -> Self {
        Self {
            planned_actions: VecDeque::new(),
            previous_position: None,
            previous_shape: None,
            previous_action: None,
        }
    }

    fn choose_action(&mut self, state: &BoardState) -> TetrisAction {
        let position = (
            state.tetromino_row,
            state.tetromino_col,
            state.tetromino_rotation,
        );

        let mut should_replan = self.planned_actions.is_empty();

        // A different shape means a new piece.
        if self.previous_shape != Some(state.tetromino) {
            should_replan = true;
        }

        // A newly spawned piece usually moves upward.
        if let Some((previous_row, _, _)) = self.previous_position {
            if state.tetromino_row > previous_row {
                should_replan = true;
            }
        }

        // Check whether the last movement actually happened.
        if let (Some(previous), Some(action)) = (self.previous_position, self.previous_action) {
            match action {
                TetrisAction::L if position.1 == previous.1 => {
                    should_replan = true;
                }
                TetrisAction::R if position.1 == previous.1 => {
                    should_replan = true;
                }
                TetrisAction::C | TetrisAction::W if position.2 == previous.2 => {
                    should_replan = true;
                }
                _ => {}
            }
        }

        if should_replan {
            self.planned_actions = find_best_actions(state).into_iter().collect();
        }

        let action = self.planned_actions.pop_front().unwrap_or(TetrisAction::H);

        self.previous_position = Some(position);
        self.previous_shape = Some(state.tetromino);
        self.previous_action = Some(action);

        // Hard drop locks the piece, so the next call needs a new plan.
        if action == TetrisAction::H {
            self.planned_actions.clear();
            self.previous_position = None;
            self.previous_shape = None;
            self.previous_action = None;
        }

        action
    }
}

impl SinglePlayerBot for GoodBot {
    fn new() -> Self {
        Self::create()
    }

    fn get_move(&mut self, state: &BoardState) -> TetrisAction {
        self.choose_action(state)
    }
}

impl MultiPlayerBot for GoodBot {
    fn new() -> Self {
        Self::create()
    }

    fn get_move(
        &mut self,
        p1_state: &BoardState,
        _p1_garbage: &[GarbageLine],
        _p2_state: &BoardState,
        _p2_garbage: &[GarbageLine],
    ) -> TetrisAction {
        self.choose_action(p1_state)
    }
}

// Your existing Game::step expects input::TetrisAction.
impl From<TetrisAction> for crate::input::TetrisAction {
    fn from(action: TetrisAction) -> Self {
        match action {
            TetrisAction::C => Self::C,
            TetrisAction::W => Self::W,
            TetrisAction::L => Self::L,
            TetrisAction::R => Self::R,
            TetrisAction::D => Self::D,
            TetrisAction::H => Self::H,
            TetrisAction::E => Self::E,
        }
    }
}
