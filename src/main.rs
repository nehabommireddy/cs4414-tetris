use clap::{ArgAction, Parser};
use p5_tetris::bot_runner::{BotRunner, BOT_NAMES};
use p5_tetris::game::Game;
use p5_tetris::input::parse_inputs;
use p5_tetris::multiplayer::Multiplayer;
use p5_tetris::output::{multi_print_boards, print_board};
use p5_tetris::resources::Resources;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
struct Cli {
    #[arg(long)]
    simulate_inputs_single_player: Option<PathBuf>,

    #[arg(long, action = ArgAction::Append, num_args = 1)]
    simulate_inputs_multi_player: Vec<PathBuf>,

    #[arg(long)]
    list_bots: bool,

    #[arg(long)]
    trace_bot_run: bool,

    #[arg(value_name = "TRACE_ARG", num_args = 0..=4)]
    legacy_trace_args: Vec<String>,

    #[arg(long)]
    p1: Option<String>,

    #[arg(long)]
    p2: Option<String>,

    #[arg(long)]
    o1: Option<PathBuf>,

    #[arg(long)]
    o2: Option<PathBuf>,

    #[arg(long)]
    resources: Option<PathBuf>,
}

fn main() {
    if let Err(error) = run(Cli::parse()) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run(args: Cli) -> Result<(), String> {
    let mode_count = [
        args.simulate_inputs_single_player.is_some(),
        !args.simulate_inputs_multi_player.is_empty(),
        args.list_bots,
        args.trace_bot_run,
    ]
    .into_iter()
    .filter(|selected| *selected)
    .count();
    if mode_count != 1 {
        return Err(
            "Choose one of the following modes: --simulate-inputs-single-player, --simulate-inputs-multi-player, --list-bots, or --trace-bot-run".to_owned(),
        );
    }

    if args.list_bots {
        for &name in BOT_NAMES {
            println!("{name}");
        }
        return Ok(());
    }

    if let Some(input_path) = args.simulate_inputs_single_player.as_deref() {
        let resources = load_resources(args.resources.as_deref())?;
        let actions =
            parse_inputs(input_path).map_err(|error| format!("Error parsing inputs: {error}"))?;
        let mut game =
            Game::new(&resources).map_err(|error| format!("Error starting game: {error}"))?;
        game.run(&actions)
            .map_err(|error| format!("Game simulation error: {error}"))?;
        print_board(&game.render());
        return Ok(());
    }

    if !args.simulate_inputs_multi_player.is_empty() {
        if args.simulate_inputs_multi_player.len() != 2 {
            return Err("--simulate-inputs-multi-player must be supplied exactly twice".to_owned());
        }

        let resources = load_resources(args.resources.as_deref())?;
        let actions1 = parse_inputs(&args.simulate_inputs_multi_player[0])
            .map_err(|error| format!("Error parsing player 1 inputs: {error}"))?;
        let actions2 = parse_inputs(&args.simulate_inputs_multi_player[1])
            .map_err(|error| format!("Error parsing player 2 inputs: {error}"))?;
        let mut game = Multiplayer::new(&resources)
            .map_err(|error| format!("Error starting game: {error}"))?;
        game.run(&actions1, &actions2)
            .map_err(|error| format!("Game simulation failed: {error}"))?;
        multi_print_boards(&game.p1().render(), &game.p2().render());
        return Ok(());
    }

    if args.trace_bot_run {
        let named_args_present =args.p1.is_some() || args.p2.is_some() || args.o1.is_some() || args.o2.is_some();
        let (bot_name1, trace1, bot_name2, trace2) = if named_args_present {
            if !args.legacy_trace_args.is_empty() {
                return Err("Do not combine trace options with rest".to_owned());
            }
            let (Some(bot_name1), Some(bot_name2), Some(trace1), Some(trace2)) = (
                args.p1.as_deref(),args.p2.as_deref(),args.o1.as_deref(),args.o2.as_deref(),
            ) else {
                return Err("requires --p1, --p2, --o1, --o2 ".to_owned(),);
            };
            (bot_name1, trace1, bot_name2, trace2)
        } else {
            if args.legacy_trace_args.len() != 4 {
                return Err("requires --p1, --p2, --o1, and --o2 (or four legacy positional arguments)".to_owned(),);
            }
            (args.legacy_trace_args[0].as_str(),Path::new(&args.legacy_trace_args[1]),args.legacy_trace_args[2].as_str(),Path::new(&args.legacy_trace_args[3]),)
        };

        let mut runner = BotRunner::new(bot_name1, bot_name2)?;
        let resources = load_resources(args.resources.as_deref())?;
        let result = runner.run(&resources)?;

        fs::write(trace1, result.p1_trace)
            .map_err(|error| format!("Writing trace for player 1 failed: {error}"))?;
        fs::write(trace2, result.p2_trace)
            .map_err(|error| format!("Writing trace for player 2 failed: {error}"))?;
        multi_print_boards(&result.p1_board, &result.p2_board);
        return Ok(());
    }

    Err("Select a mode".to_owned())
}

fn load_resources(path: Option<&Path>) -> Result<Resources, String> {
    let path = path.unwrap_or_else(|| Path::new("resources"));
    Resources::load(path).map_err(|error| format!("Resource loading error: {error}"))
}
