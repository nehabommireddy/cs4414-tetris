use p5_tetris::bot::{GoodBot, SinglePlayerBot};
use p5_tetris::game::Game;
use p5_tetris::resources::Resources;

use std::path::Path;
use std::time::Instant;

#[test]
fn good_bot_clears_40_lines() {
    let resources = Resources::load(Path::new("resources")).expect("Failed to load resources");

    let mut game = Game::new(&resources).expect("Failed to create game");

    let mut bot = <GoodBot as SinglePlayerBot>::new();

    let start_time = Instant::now();
    let mut frames = 0;

    const MAX_FRAMES: usize = 100_000;

    while !game.topped_out() && game.lines_cleared() < 40 && frames < MAX_FRAMES {
        let state = game.board_state();
        let action = bot.get_move(&state);

        game.step(action.into()).expect("Failed to advance game");

        frames += 1;
    }

    println!("Frames: {}", frames);
    println!("Lines cleared: {}", game.lines_cleared());
    println!("Topped out: {}", game.topped_out());
    println!("Elapsed: {:?}", start_time.elapsed());

    assert!(
        game.lines_cleared() >= 40,
        "GoodBot failed to clear 40 lines"
    );
}
