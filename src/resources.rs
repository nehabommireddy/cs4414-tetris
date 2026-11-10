use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub width: usize,
    pub height: usize,
    pub gravity: u64,
    pub gravity_decrease: u64,
    pub lock_delay: u64,
    pub random_seed: [u8; 16],
}

#[derive(Debug, Deserialize)]
pub struct TetrominoData {
    pub states: [[[i32; 2]; 4]; 4],
    pub wall_kick: String,
}

pub type WallKickTable = HashMap<String, HashMap<String, Vec<[i32; 2]>>>;

#[derive(Debug)]
pub struct Resources {
    pub config: Config,
    pub tetrominoes: HashMap<String, TetrominoData>,
    pub wall_kicks: WallKickTable,
}

fn load_json<T: DeserializeOwned>(path: &Path) -> Result<T, Box<dyn Error>> {
    let contents = fs::read_to_string(path)?;
    let data = serde_json::from_str::<T>(&contents)?;
    Ok(data)
}

impl Resources {
    pub fn load(dir: &Path) -> Result<Self, Box<dyn Error>> {
        let config = load_json(&dir.join("config.json"))?;
        let tetrominoes = load_json(&dir.join("tetrominoes.json"))?;
        let wall_kicks = load_json(&dir.join("wall_kicks.json"))?;

        Ok(Self {
            config,
            tetrominoes,
            wall_kicks,
        })
    }
}
