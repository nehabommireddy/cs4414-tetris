use std::collections::VecDeque;

use rand::Rng;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro128PlusPlus;

use crate::bot::GarbageLine;
use crate::game::{Game, TickOutcome};
use crate::input::TetrisAction;
use crate::resources::Resources;

/// A pending group of garbage lines heading toward one player. All lines in the group
/// share the same hole column (the column is only generated the moment the group is
/// applied to the board, in keeping with the spec).
#[derive(Debug, Clone, Copy)]
struct GarbageGroup {
    rows: usize,
    ticks_till_applied: usize,
}

impl GarbageGroup {
    fn new(rows: usize, ticks_till_applied: usize) -> Self {
        Self {
            rows,
            ticks_till_applied,
        }
    }
}

/// Two-player simulator. Owns both `Game`s and the per-player garbage queues.
pub struct Multiplayer<'a> {
    p1: Game<'a>,
    p2: Game<'a>,
    /// Garbage waiting to be applied to player 1 (sent by player 2).
    p1_queue: VecDeque<GarbageGroup>,
    /// Garbage waiting to be applied to player 2 (sent by player 1).
    p2_queue: VecDeque<GarbageGroup>,
    p1_garbage_rng: Xoshiro128PlusPlus,
    p2_garbage_rng: Xoshiro128PlusPlus,
    garbage_queue_time: usize,
    board_width: usize,
}

impl<'a> Multiplayer<'a> {
    pub fn new(resources: &'a Resources) -> Result<Self, String> {
        let p1 = Game::new(resources)?;
        let p2 = Game::new(resources)?;
        let seed = resources.config.random_seed;
        Ok(Self {
            p1,
            p2,
            p1_queue: VecDeque::new(),
            p2_queue: VecDeque::new(),
            p1_garbage_rng: Xoshiro128PlusPlus::from_seed(seed),
            p2_garbage_rng: Xoshiro128PlusPlus::from_seed(seed),
            garbage_queue_time: resources.config.garbage_queue_time,
            board_width: resources.config.width,
        })
    }

    pub fn p1(&self) -> &Game<'a> {
        &self.p1
    }

    pub fn p2(&self) -> &Game<'a> {
        &self.p2
    }

    pub fn p1_topped_out(&self) -> bool {
        self.p1.topped_out()
    }

    pub fn p2_topped_out(&self) -> bool {
        self.p2.topped_out()
    }

    /// Returns true iff at least one player has topped out.
    pub fn game_over(&self) -> bool {
        self.p1_topped_out() || self.p2_topped_out()
    }

    /// Snapshot of player 1's pending garbage queue in the format the bot API expects.
    pub fn p1_garbage(&self) -> Vec<GarbageLine> {
        garbage_snapshot(&self.p1_queue)
    }

    /// Snapshot of player 2's pending garbage queue in the format the bot API expects.
    pub fn p2_garbage(&self) -> Vec<GarbageLine> {
        garbage_snapshot(&self.p2_queue)
    }

    /// Advances both boards by one frame with the given per-player actions. Returns the
    /// per-player tick outcomes (lines cleared, locked, topped out).
    pub fn step(
        &mut self,
        p1_action: TetrisAction,
        p2_action: TetrisAction,
    ) -> Result<(TickOutcome, TickOutcome), String> {
        let p1_outcome = self.p1.step_with_outcome(p1_action)?;
        let p2_outcome = self.p2.step_with_outcome(p2_action)?;

        self.flow_garbage(p1_outcome.lines_cleared, p2_outcome.lines_cleared);

        if p1_outcome.locked {
            self.apply_due_garbage_p1()?;
        }
        if p2_outcome.locked {
            self.apply_due_garbage_p2()?;
        }

        // Timers tick down after everything else on this frame.
        decrement_timers(&mut self.p1_queue);
        decrement_timers(&mut self.p2_queue);

        Ok((p1_outcome, p2_outcome))
    }

    /// Per-tick garbage flow (spec steps 3-5): compute sent lines, cancel against the
    /// sender's own pending queue, enqueue what remains on the receiver's queue.
    ///
    /// Important: cancellation is against the sender's *existing* queue, not against
    /// garbage added this very tick, so we take snapshots of queue lengths first.
    fn flow_garbage(&mut self, p1_cleared: usize, p2_cleared: usize) {
        let s1 = p1_cleared.saturating_sub(1);
        let s2 = p2_cleared.saturating_sub(1);
        let g1 = self.p1_queue.len();
        let g2 = self.p2_queue.len();

        let s1_to_p2 = s1.saturating_sub(g1);
        let p1_cancels = s1.min(g1);
        let s2_to_p1 = s2.saturating_sub(g2);
        let p2_cancels = s2.min(g2);

        for _ in 0..p1_cancels {
            self.p1_queue.pop_front();
        }
        for _ in 0..p2_cancels {
            self.p2_queue.pop_front();
        }

        if s1_to_p2 > 0 {
            self.p2_queue
                .push_back(GarbageGroup::new(s1_to_p2, self.garbage_queue_time));
        }
        if s2_to_p1 > 0 {
            self.p1_queue
                .push_back(GarbageGroup::new(s2_to_p1, self.garbage_queue_time));
        }
    }

    fn apply_due_garbage_p1(&mut self) -> Result<(), String> {
        if self.p1.topped_out() {
            return Ok(());
        }
        while let Some(group) = self.p1_queue.front().copied() {
            if group.ticks_till_applied != 0 {
                break;
            }
            self.p1_queue.pop_front();
            let hole_col = (self.p1_garbage_rng.next_u32() as usize) % self.board_width;
            let topped = self.p1.board_mut().add_garbage(group.rows, hole_col)?;
            if topped {
                self.p1.force_top_out();
                return Ok(());
            }
        }
        Ok(())
    }

    fn apply_due_garbage_p2(&mut self) -> Result<(), String> {
        if self.p2.topped_out() {
            return Ok(());
        }
        while let Some(group) = self.p2_queue.front().copied() {
            if group.ticks_till_applied != 0 {
                break;
            }
            self.p2_queue.pop_front();
            let hole_col = (self.p2_garbage_rng.next_u32() as usize) % self.board_width;
            let topped = self.p2.board_mut().add_garbage(group.rows, hole_col)?;
            if topped {
                self.p2.force_top_out();
                return Ok(());
            }
        }
        Ok(())
    }

    /// Runs the simulator until both input lists are exhausted or either player tops out.
    /// Shorter action lists are padded with `TetrisAction::E`.
    pub fn run(
        &mut self,
        p1_actions: &[TetrisAction],
        p2_actions: &[TetrisAction],
    ) -> Result<(), String> {
        let total = p1_actions.len().max(p2_actions.len());
        for i in 0..total {
            if self.game_over() {
                break;
            }
            let a1 = p1_actions.get(i).copied().unwrap_or(TetrisAction::E);
            let a2 = p2_actions.get(i).copied().unwrap_or(TetrisAction::E);
            self.step(a1, a2)?;
        }
        Ok(())
    }
}

fn garbage_snapshot(queue: &VecDeque<GarbageGroup>) -> Vec<GarbageLine> {
    queue
        .iter()
        .map(|g| GarbageLine {
            num_rows_to_create: g.rows,
            ticks_till_applied: g.ticks_till_applied,
        })
        .collect()
}

fn decrement_timers(queue: &mut VecDeque<GarbageGroup>) {
    for group in queue.iter_mut() {
        if group.ticks_till_applied > 0 {
            group.ticks_till_applied -= 1;
        }
    }
}
