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

#[test]
fn check_good_bot_won() {
    use p5_tetris::input::parse_inputs;
    use p5_tetris::multiplayer::Multiplayer;
    use p5_tetris::resources::Resources;
    use std::path::Path;
    use std::process::Command;

    let resources_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("resources");
    let resources = Resources::load(&resources_path).expect("load resources");
    let temp_dir = std::env::temp_dir();
    let trace1 = temp_dir.join(format!("p5-tetris-p1-{}.txt", std::process::id()));
    let trace2 = temp_dir.join(format!("p5-tetris-p2-{}.txt", std::process::id()));

    let output = Command::new(env!("CARGO_BIN_EXE_p5_tetris"))
        .arg("--resources")
        .arg(&resources_path)
        .args(["--trace-bot-run", "--p1", "good-bot", "--p2", "bad-bot"])
        .arg("--o1")
        .arg(&trace1)
        .arg("--o2")
        .arg(&trace2)
        .output()
        .expect("run bot matchup");
    assert!(
        output.status.success(),
        "bot matchup failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let actions1 = parse_inputs(&trace1).expect("parse player 1 trace");
    let actions2 = parse_inputs(&trace2).expect("parse player 2 trace");
    let mut game = Multiplayer::new(&resources).expect("create multiplayer game");
    game.run(&actions1, &actions2).expect("replay bot traces");
    assert!(!game.p1_topped_out(), "good-bot topped out");
    assert!(game.p2_topped_out(), "bad-bot did not top out");

    std::fs::remove_file(trace1).expect("remove player 1 trace");
    std::fs::remove_file(trace2).expect("remove player 2 trace");
}
