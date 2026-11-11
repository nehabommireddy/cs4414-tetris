use crate::resources::WallKickTable;
use crate::tetromino::Tetromino;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    width: usize,
    height: usize,
    cells: Vec<Vec<bool>>,
}

impl Board {
    pub fn new(width: usize, height: usize) -> Result<Self, String> {
        if width == 0 || height == 0 {
            return Err("Dimensions must be greater than zero".to_owned());
        }

        let storage_height = height.checked_add(1).ok_or_else(|| "Board height is too large".to_owned())?;
        Ok(Self {width, height, cells: vec![vec![false; width]; storage_height]})
    }

    pub const fn width(&self) -> usize {
        self.width
    }

    pub const fn height(&self) -> usize {
        self.height
    }

    pub fn is_full(&self, col: usize, row: usize) -> Option<bool> {
        self.cells.get(row).and_then(|line| line.get(col)).copied()
    }

    pub fn placeable(&self, piece: &Tetromino) -> bool {
        let Some(cells) = piece.cells() else {
            return false;
        };
        cells.into_iter().all(|(col, row)| {
            if col < 0 || col as usize >= self.width || row < 0 {
                return false;
            }

            row as usize >= self.cells.len() || !self.cells[row as usize][col as usize]
        })
    }

    pub fn try_move(&self, piece: &mut Tetromino, d_col: i32, d_row: i32) -> bool {
        let Some(candidate) = offset_piece(piece, d_col, d_row) else {
            return false;
        };
        if self.placeable(&candidate) {
            *piece = candidate;
            true
        } else {
            false
        }
    }

    pub fn moveable(&self, piece: &Tetromino, d_col: i32, d_row: i32) -> bool {
        offset_piece(piece, d_col, d_row).is_some_and(|candidate| self.placeable(&candidate))
    }

    pub fn try_rotate(&self, piece: &mut Tetromino, clockwise: bool, wall_kicks: &WallKickTable) -> Result<bool, String> {
        let from = piece.rotation();
        let to = if clockwise {(from + 1) % 4} else {(from + 3) % 4};
        let states = ["o", "r", "t", "l"];
        let table = wall_kicks.get(piece.wall_kick()).ok_or_else(|| format!("Missing kick table for {}", piece.wall_kick()))?;
        let from_kicks = table.get(states[from]).ok_or_else(|| format!("Missing kick state {}", states[from]))?;
        let to_kicks = table.get(states[to]).ok_or_else(|| format!("Missing kick state {}", states[to]))?;
        if from_kicks.len() != to_kicks.len() {
            return Err(format!("Mismatched kick table lengths for {} rotation", piece.shape()));
        }

        for (&[from_x, from_y], &[to_x, to_y]) in from_kicks.iter().zip(to_kicks) {
            let Some(d_col) = from_x.checked_sub(to_x) else {
                continue;
            };
            let Some(d_row) = from_y.checked_sub(to_y) else {
                continue;
            };
            let Some(mut candidate) = offset_piece(piece, d_col, d_row) else {
                continue;
            };
            candidate.set_rotation(to)?;
            if self.placeable(&candidate) {
                *piece = candidate;
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn is_top_out(&self, piece: &Tetromino) -> bool {
        piece.cells().is_none_or(|cells| {cells.into_iter().any(|(_, row)| row >= 0 && row as usize > self.height)})
    }

    pub fn lock_piece(&mut self, piece: &Tetromino) -> Result<(), String> {
        if self.is_top_out(piece) {
            return Err("Cannot lock a top-out tetromino; check is_top_out first".to_owned());
        }
        if !self.placeable(piece) {
            return Err("Cannot lock colliding tetromino".to_owned());
        }

        let cells = piece.cells().ok_or_else(|| "Tetromino coordinates overflow".to_owned())?;
        if cells.iter().any(|&(col, row)| {
            col < 0 || col as usize >= self.width || row < 0 || row as usize >= self.cells.len()
        }) {
            return Err("Cannot lock tetromino outside board".to_owned());
        }

        for (col, row) in cells {
            self.cells[row as usize][col as usize] = true;
        }
        Ok(())
    }

    pub fn clear_lines(&mut self) -> usize {
        let previous_height = self.cells.len();
        self.cells.retain(|row| !row.iter().all(|&occupied| occupied));
        let cleared = previous_height - self.cells.len();
        self.cells.resize_with(self.height + 1, || vec![false; self.width]);
        cleared
    }

    pub fn render(&self, piece: Option<&Tetromino>, rows: usize) -> Vec<Vec<bool>> {
        let mut rendered = vec![vec![false; self.width]; rows];
        for (row, cells) in self.cells.iter().enumerate().take(rows) {
            rendered[rows - 1 - row].copy_from_slice(cells);
        }

        if let Some(piece) = piece {
            if let Some(cells) = piece.cells() {
                for (col, row) in cells {
                    if col >= 0 && (col as usize) < self.width && row >= 0 && (row as usize) < rows
                    {
                        rendered[rows - 1 - row as usize][col as usize] = true;
                    }
                }
            }
        }

        rendered
    }
}

fn offset_piece(piece: &Tetromino, d_col: i32, d_row: i32) -> Option<Tetromino> {
    let mut candidate = piece.clone();
    candidate.col = candidate.col.checked_add(d_col)?;
    candidate.row = candidate.row.checked_add(d_row)?;
    Some(candidate)
}
