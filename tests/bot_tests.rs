use p5_tetris::bot::{
    BoardState, CellState, GoodBot, MultiPlayerBot, SinglePlayerBot, TetrominoType,
};

fn empty_state() -> BoardState {
    BoardState {
        tetromino_row: 20,
        tetromino_col: 4,
        tetromino_rotation: 0,
        tetromino: TetrominoType::T,
        board: [[CellState::Empty; 10]; 20],
    }
}

#[test]
fn single_player_bot_returns_action() {
    let mut bot = <GoodBot as SinglePlayerBot>::new();
    let state = empty_state();

    let _action = SinglePlayerBot::get_move(&mut bot, &state);
}

#[test]
fn multiplayer_bot_returns_action() {
    let mut bot = <GoodBot as MultiPlayerBot>::new();
    let state = empty_state();

    let _action = MultiPlayerBot::get_move(&mut bot, &state, &[], &state, &[]);
}
