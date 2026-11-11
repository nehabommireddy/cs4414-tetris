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

fn load_json<T: DeserializeOwned>(
    file_path: &Path,
) -> Result<T, Box<dyn Error>> {
    let file_contents = fs::read_to_string(file_path)?;
    let parsed_data = serde_json::from_str(&file_contents)?;

    Ok(parsed_data)
}

impl Resources {
    pub fn load(resources_directory: &Path) -> Result<Self, Box<dyn Error>> {
        let config_path = resources_directory.join("config.json");
        let tetrominoes_path = resources_directory.join("tetrominoes.json");
        let wall_kicks_path = resources_directory.join("wall_kicks.json");

        let config = load_json(&config_path)?;
        let tetrominoes = load_json(&tetrominoes_path)?;
        let wall_kicks = load_json(&wall_kicks_path)?;

        Ok(Resources {
            config,
            tetrominoes,
            wall_kicks,
        })
    }
}