use crate::bot::{
    BadBot, BoardState, GarbageLine, GoodBot, MultiPlayerBot, TetrisAction as BotAction,
};
use crate::input::TetrisAction as InputAction;
use crate::multiplayer::Multiplayer;
use crate::resources::Resources;

pub const BOT_NAMES: &[&str] = &["good-bot", "bad-bot"];

enum BotInstance {
    Good(GoodBot),
    Bad(BadBot),
}

impl BotInstance {
    fn from_name(bot_name: &str) -> Result<Self, String> {
        match bot_name {
            "good-bot" => Ok(Self::Good(<GoodBot as MultiPlayerBot>::new())),
            "bad-bot" => Ok(Self::Bad(<BadBot as MultiPlayerBot>::new())),
            _ => Err(format!("Bot name is not valid: {bot_name}")),
        }
    }

    fn get_move(
        &mut self,
        curr_state: &BoardState,
        curr_garbage: &[GarbageLine],
        opp_state: &BoardState,
        opp_garbage: &[GarbageLine],
    ) -> BotAction {
        match self {
            Self::Good(bot) => bot.get_move(curr_state, curr_garbage, opp_state, opp_garbage),
            Self::Bad(bot) => bot.get_move(curr_state, curr_garbage, opp_state, opp_garbage),
        }
    }
}

pub struct BotRunResult {
    pub p1_board: Vec<Vec<bool>>,
    pub p2_board: Vec<Vec<bool>>,
    pub p1_trace: String,
    pub p2_trace: String,
}

pub struct BotRunner {
    bot1: BotInstance,
    bot2: BotInstance,
}

impl BotRunner {
    pub fn new(bot1_name: &str, bot2_name: &str) -> Result<Self, String> {
        Ok(Self {
            bot1: BotInstance::from_name(bot1_name)?,
            bot2: BotInstance::from_name(bot2_name)?,
        })
    }

    pub fn run(&mut self, resources: &Resources) -> Result<BotRunResult, String> {
        let mut game = Multiplayer::new(resources)?;
        let mut p1_trace = String::new();
        let mut p2_trace = String::new();

        while !game.game_over() {
            let p1_state = game.p1().board_state();
            let p2_state = game.p2().board_state();
            let p1_garbage = game.p1_garbage();
            let p2_garbage = game.p2_garbage();

            let p1_action = self
                .bot1
                .get_move(&p1_state, &p1_garbage, &p2_state, &p2_garbage);
            let p2_action = self
                .bot2
                .get_move(&p2_state, &p2_garbage, &p1_state, &p1_garbage);

            p1_trace.push(action_to_char(p1_action));
            p2_trace.push(action_to_char(p2_action));

            game.step(InputAction::from(p1_action), InputAction::from(p2_action))?;
        }

        Ok(BotRunResult {
            p1_board: game.p1().render(),
            p2_board: game.p2().render(),
            p1_trace,
            p2_trace,
        })
    }
}

fn action_to_char(action: BotAction) -> char {
    match action {
        BotAction::C => 'C',
        BotAction::W => 'W',
        BotAction::L => 'L',
        BotAction::R => 'R',
        BotAction::D => 'D',
        BotAction::H => 'H',
        BotAction::E => 'E',
    }
}
