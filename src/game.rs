use crate::board::Board;
use crate::bot::{BoardState, CellState, TetrominoType};
use crate::input::TetrisAction;
use crate::randomizer::BagRandomizer;
use crate::resources::Resources;
use crate::tetromino::{Tetromino, TetrominoShape};

/// Number of rows printed: the 20 visible rows plus 2 above for the spawn area.
const RENDER_ROWS: usize = 22;

/// Result of a single game tick. Used by the multiplayer driver to drive garbage flow.
#[derive(Debug, Clone, Copy, Default)]
pub struct TickOutcome {
    /// Number of lines the player cleared on this tick.
    pub lines_cleared: usize,
    /// Whether the current piece locked on this tick (hard drop or lock delay expired).
    pub locked: bool,
    /// Whether the player topped out on this tick.
    pub topped_out: bool,
}

pub struct Game<'a> {
    resources: &'a Resources,
    board: Board,
    randomizer: BagRandomizer<TetrominoShape>,
    current: Tetromino,
    /// Frames since the current piece was last moved down a row by gravity.
    frames_since_drop: u64,
    topped_out: bool,
    lines_cleared: usize,
    /// Lines cleared during the most recent lock. Reset at the start of each step.
    last_lines_cleared: usize,
    /// Whether the current piece locked during the most recent step.
    last_locked: bool,
}

impl<'a> Game<'a> {
    pub fn new(resources: &'a Resources) -> Result<Self, String> {
        let config = &resources.config;
        let board = Board::new(config.width, config.height)?;
        let randomizer =
            BagRandomizer::from_parts(TetrominoShape::ALL.into_iter(), config.random_seed);
        let current = Self::make_piece(*randomizer.cur(), resources)?;

        Ok(Self {
            resources,
            board,
            randomizer,
            current,
            frames_since_drop: 0,
            topped_out: false,
            lines_cleared: 0,
            last_lines_cleared: 0,
            last_locked: false,
        })
    }

    /// Builds a piece of the given shape at the spawn position, column (width - 1) / 2, row height.
    fn make_piece(shape: TetrominoShape, resources: &Resources) -> Result<Tetromino, String> {
        let config = &resources.config;
        let col = ((config.width - 1) / 2) as i32;
        let row = config.height as i32;
        Tetromino::new(shape, row, col, resources)
    }

    /// Draws the next piece from the 7-bag and makes it the current piece.
    fn spawn(&mut self) -> Result<(), String> {
        self.randomizer.next();
        self.current = Self::make_piece(*self.randomizer.cur(), self.resources)?;
        self.frames_since_drop = 0;
        Ok(())
    }

    /// Locks the current piece. Checks for top out before clearing lines, then spawns the next piece.
    fn lock(&mut self) -> Result<(), String> {
        self.last_locked = true;
        if self.board.is_top_out(&self.current) {
            self.topped_out = true;
            return Ok(());
        }
        self.board.lock_piece(&self.current)?;
        let cleared = self.board.clear_lines();
        self.last_lines_cleared = cleared;
        self.lines_cleared += cleared;
        self.spawn()
    }

    /// Advances the game by one frame using the given input.
    pub fn step(&mut self, action: TetrisAction) -> Result<(), String> {
        self.step_with_outcome(action).map(|_| ())
    }

    /// Advances the game by one frame and returns what happened on this tick.
    /// Multiplayer code uses the returned outcome to flow garbage and detect locks.
    pub fn step_with_outcome(&mut self, action: TetrisAction) -> Result<TickOutcome, String> {
        self.last_lines_cleared = 0;
        self.last_locked = false;

        if self.topped_out {
            return Ok(TickOutcome {
                lines_cleared: 0,
                locked: false,
                topped_out: true,
            });
        }

        let config = &self.resources.config;
        let mut gravity = config.gravity;

        // 1 and 2: read and perform the action.
        match action {
            TetrisAction::L => {
                self.board.try_move(&mut self.current, -1, 0);
            }
            TetrisAction::R => {
                self.board.try_move(&mut self.current, 1, 0);
            }
            TetrisAction::C => {
                self.board
                    .try_rotate(&mut self.current, true, &self.resources.wall_kicks)?;
            }
            TetrisAction::W => {
                self.board
                    .try_rotate(&mut self.current, false, &self.resources.wall_kicks)?;
            }
            TetrisAction::D => {
                // Soft drop: gravity is lower for this frame only.
                gravity = gravity.saturating_sub(config.gravity_decrease);
            }
            TetrisAction::H => {
                while self.board.try_move(&mut self.current, 0, -1) {}
                self.lock()?;
                return Ok(self.make_outcome());
            }
            TetrisAction::E => {}
        }

        // 3: gravity and lock delay.
        self.frames_since_drop += 1;
        if self.frames_since_drop >= gravity && self.board.try_move(&mut self.current, 0, -1) {
            self.frames_since_drop = 0;
        } else if self.frames_since_drop >= config.lock_delay
            && !self.board.moveable(&self.current, 0, -1)
        {
            // 4 and 5: lock, line clears and spawning happen inside lock().
            self.lock()?;
        }

        Ok(self.make_outcome())
    }

    fn make_outcome(&self) -> TickOutcome {
        TickOutcome {
            lines_cleared: self.last_lines_cleared,
            locked: self.last_locked,
            topped_out: self.topped_out,
        }
    }

    /// Mutable access to the board for the multiplayer driver (garbage application).
    pub fn board_mut(&mut self) -> &mut Board {
        &mut self.board
    }

    /// Read-only access to the board.
    pub fn board(&self) -> &Board {
        &self.board
    }

    /// Forces the game into the topped-out state. Called by the multiplayer driver when
    /// garbage pushes the stack past row 20.
    pub fn force_top_out(&mut self) {
        self.topped_out = true;
    }

    /// Runs every action in order, stopping early if the player tops out.
    pub fn run(&mut self, actions: &[TetrisAction]) -> Result<(), String> {
        for &action in actions {
            if self.topped_out {
                break;
            }
            self.step(action)?;
        }
        Ok(())
    }

    pub fn topped_out(&self) -> bool {
        self.topped_out
    }

    /// The board as rows of filled/empty cells (top row first), with the falling piece drawn in.
    /// After a top out, the piece that topped out is not drawn and no new piece is spawned.
    pub fn render(&self) -> Vec<Vec<bool>> {
        let piece = if self.topped_out {
            None
        } else {
            Some(&self.current)
        };
        self.board.render(piece, RENDER_ROWS)
    }

    pub fn lines_cleared(&self) -> usize {
        self.lines_cleared
    }

    pub fn board_state(&self) -> BoardState {
        let tetromino = match self.current.shape() {
            TetrominoShape::I => TetrominoType::I,
            TetrominoShape::J => TetrominoType::J,
            TetrominoShape::L => TetrominoType::L,
            TetrominoShape::O => TetrominoType::O,
            TetrominoShape::S => TetrominoType::S,
            TetrominoShape::T => TetrominoType::T,
            TetrominoShape::Z => TetrominoType::Z,
        };

        let mut board = [[CellState::Empty; 10]; 20];

        for row in 0..20 {
            for col in 0..10 {
                if self.board.is_full(col, row) == Some(true) {
                    board[row][col] = CellState::Filled;
                }
            }
        }

        BoardState {
            tetromino_row: self.current.row as usize,
            tetromino_col: self.current.col as usize,
            tetromino_rotation: self.current.rotation(),
            tetromino,
            board,
        }
    }
}
