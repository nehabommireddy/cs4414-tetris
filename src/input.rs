use std::fs;
use std::path::Path;

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

pub fn parse_inputs(path: &Path) -> Result<Vec<TetrisAction>, String> {
    let contents = fs::read_to_string(path).map_err(|e| e.to_string())?;

    let mut actions = Vec::new();

    for ch in contents.chars() {
        let action = match ch {
            'C' => TetrisAction::C,
            'W' => TetrisAction::W,
            'L' => TetrisAction::L,
            'R' => TetrisAction::R,
            'D' => TetrisAction::D,
            'H' => TetrisAction::H,
            'E' => TetrisAction::E,
            c if c.is_ascii_whitespace() => continue,
            _ => return Err(format!("Invalid action: {}", ch)),
        };
        actions.push(action)
    }
    Ok(actions)
}
