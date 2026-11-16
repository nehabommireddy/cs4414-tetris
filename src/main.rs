use clap::Parser;
use p5_tetris::game::Game;
use p5_tetris::input::parse_inputs;
use p5_tetris::output::print_board;
use p5_tetris::resources::Resources;
use std::path::PathBuf;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(long)]
    simulate_inputs_single_player: PathBuf,

    #[arg(long)]
    resources: PathBuf,
}

fn main() {
    let args = Cli::parse();
    let resources = Resources::load(&args.resources).expect("Failed to load resources");
    let actions =
        parse_inputs(&args.simulate_inputs_single_player).expect("Failed to parse inputs");

    let mut game = Game::new(&resources).expect("Failed to start game");
    game.run(&actions).expect("Game simulation failed");
    print_board(&game.render());
}
