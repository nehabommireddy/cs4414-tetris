use p5_tetris::bot::{BoardState, Bot, CellState, MultiPlayerBot, TetrominoType};

pub type MySinglePlayerBot = p5_tetris::bot::MySinglePlayerBot;
pub type MyMultiPlayerBot = p5_tetris::bot::MyMultiPlayerBot;
pub type MyMultiplayerBot = p5_tetris::bot::MyMultiplayerBot;

fn empty_state() -> BoardState {
    BoardState {
        tetromino_row: 20,
        tetromino_col: 4,
        tetromino_rotation: 0,
        tetromino: TetrominoType::J,
        board: [[CellState::Empty; 10]; 20],
    }
}

#[test]
fn create_bot_and_get_any_action_test() {
    let mut bot = MySinglePlayerBot::new();
    let _action = bot.get_move(&empty_state());
}

#[test]
fn create_mut_bot_and_get_any_action_test() {
    let mut bot = MyMultiplayerBot::new();
    let state = empty_state();
    let _action = bot.get_move(&state, &[], &state, &[]);
}