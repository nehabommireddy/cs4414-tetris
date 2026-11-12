use crate::board::Board;
use crate::input::TetrisAction;
use crate::randomizer::BagRandomizer;
use crate::resources::Resources;
use crate::tetromino::{Tetromino, TetrominoShape};

/// Number of rows printed: the 20 visible rows plus 2 above for the spawn area.
const RENDER_ROWS: usize = 22;

pub struct Game<'a> {
    resources: &'a Resources,
    board: Board,
    randomizer: BagRandomizer<TetrominoShape>,
    current: Tetromino,
    /// Frames since the current piece was last moved down a row by gravity.
    frames_since_drop: u64,
    topped_out: bool,
}

impl<'a> Game<'a> {
    pub fn new(resources: &'a Resources) -> Result<Self, String> {
        let config = &resources.config;
        let board = Board::new(config.width, config.height)?;
        let randomizer = BagRandomizer::from_parts(TetrominoShape::ALL.into_iter(), config.random_seed);
        let current = Self::make_piece(*randomizer.cur(), resources)?;

        Ok(Self { resources, board, randomizer, current, frames_since_drop: 0, topped_out: false })
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
        if self.board.is_top_out(&self.current) {
            self.topped_out = true;
            return Ok(());
        }
        self.board.lock_piece(&self.current)?;
        self.board.clear_lines();
        self.spawn()
    }

    /// Advances the game by one frame using the given input.
    pub fn step(&mut self, action: TetrisAction) -> Result<(), String> {
        if self.topped_out {
            return Ok(());
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
                self.board.try_rotate(&mut self.current, true, &self.resources.wall_kicks)?;
            }
            TetrisAction::W => {
                self.board.try_rotate(&mut self.current, false, &self.resources.wall_kicks)?;
            }
            TetrisAction::D => {
                // Soft drop: gravity is lower for this frame only.
                gravity = gravity.saturating_sub(config.gravity_decrease);
            }
            TetrisAction::H => {
                while self.board.try_move(&mut self.current, 0, -1) {}
                return self.lock();
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
            return self.lock();
        }

        Ok(())
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
        let piece = if self.topped_out { None } else { Some(&self.current) };
        self.board.render(piece, RENDER_ROWS)
    }
}
