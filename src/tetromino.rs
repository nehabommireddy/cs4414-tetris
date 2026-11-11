use crate::resources::{Resources, TetrominoData};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TetrominoShape {
    I,
    J,
    L,
    O,
    S,
    T,
    Z,
}

impl TetrominoShape {
    pub const ALL: [Self; 7] = [Self::I, Self::J, Self::L, Self::O, Self::S, Self::T, Self::Z];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::I => "i",
            Self::J => "j",
            Self::L => "l",
            Self::O => "o",
            Self::S => "s",
            Self::T => "t",
            Self::Z => "z",
        }
    }

    pub fn from_str(name: &str) -> Option<Self> {
        match name {
            "i" => Some(Self::I),
            "j" => Some(Self::J),
            "l" => Some(Self::L),
            "o" => Some(Self::O),
            "s" => Some(Self::S),
            "t" => Some(Self::T),
            "z" => Some(Self::Z),
            _ => None,
        }
    }
}

impl fmt::Display for TetrominoShape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct Tetromino {
    shape: TetrominoShape,
    pub row: i32,
    pub col: i32,
    rotation: usize,
    data: TetrominoData,
}

impl Tetromino {
    pub fn new(shape: TetrominoShape, row: i32, col: i32, resources: &Resources) -> Result<Self, String> {
        let data = resources.tetrominoes.get(shape.as_str()).ok_or_else(|| format!("Missing data for {shape}"))?.clone();
        Ok(Self {shape, row, col, rotation: 0, data})
    }

    pub const fn shape(&self) -> TetrominoShape {
        self.shape
    }

    pub fn cells(&self) -> Option<[(i32, i32); 4]> {
        let mut cells = [(0, 0); 4];
        for (index, [dx, dy]) in self.data.states[self.rotation].iter().enumerate() {
            cells[index] = (self.col.checked_add(*dx)?, self.row.checked_add(*dy)?);
        }
        Some(cells)
    }

    pub const fn rotation(&self) -> usize {
        self.rotation
    }

    pub fn set_rotation(&mut self, rotation: usize) -> Result<(), String> {
        if rotation >= self.data.states.len() {
            return Err(format!("Invalid rotation: {rotation}"));
        }

        self.rotation = rotation;
        Ok(())
    }

    pub fn wall_kick(&self) -> &str {
        &self.data.wall_kick
    }
}
