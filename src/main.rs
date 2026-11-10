use clap::Parser;
use p5_tetris::input::parse_inputs;
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
    let _ = (resources, actions);
}
