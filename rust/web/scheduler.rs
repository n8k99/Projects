//! Scheduled Auto Login and Action — cron-like task scheduler.
//!
//! Tasks are **data**: (name, schedule, action, credential_ref). The
//! scheduler owns the tasks and the run history; `tick(now)` fires due
//! tasks through a pluggable dispatch function. Same abstract-transport
//! pattern as G072 File Downloader — the dispatch function is a parameter,
//! so tests use a fake executor and production plugs in a real one.
//!
//! A task's `next_fire_ms` is computed by its `Schedule`:
//!   - `EveryMs(interval)`: fires every `interval` milliseconds from start.
//!   - `AtMs(t)`: one-shot at absolute time t; disabled after firing.
//!   - `Daily(hour, minute)`: fires once per 24h at the given wall-clock
//!     time (input is ms-since-epoch; uses `/86_400_000` for day boundary).
//!
//! Catch-up policy: if the scheduler is ticked with a `now` far past a
//! task's `next_fire_ms`, the task fires **once** and reschedules. We
//! don't backfill every missed run — that would produce a thundering
//! herd after downtime. Same behaviour as cron (skip-missed, next run at
//! next natural boundary).

use std::collections::HashMap;

pub type TaskId = u64;

#[derive(Debug, Clone, Copy)]
pub enum Schedule {
    EveryMs(u64),            // interval
    AtMs(u64),               // one-shot absolute time
    Daily { hour: u8, minute: u8 },
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: TaskId,
    pub name: String,
    pub schedule: Schedule,
    pub action: String,          // opaque action descriptor, passed to dispatch
    pub credential_ref: Option<String>,
    pub next_fire_ms: u64,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunStatus {
    Succeeded,
    Failed,
}

#[derive(Debug, Clone)]
pub struct Run {
    pub task_id: TaskId,
    pub started_ms: u64,
    pub finished_ms: u64,
    pub status: RunStatus,
    pub message: String,
}

/// Dispatch context passed to the executor callback.
pub struct DispatchContext<'a> {
    pub task: &'a Task,
    pub now_ms: u64,
}

pub type DispatchResult = Result<String, String>;

pub struct Scheduler {
    tasks: Vec<Task>,
    history: Vec<Run>,
    next_id: TaskId,
}

impl Scheduler {
    pub fn new() -> Self {
        Self { tasks: Vec::new(), history: Vec::new(), next_id: 1 }
    }

    /// Add a task. `first_fire_base_ms` is the reference time for computing
    /// the task's initial `next_fire_ms` — typically "now" when scheduling.
    pub fn add_task(
        &mut self,
        name: &str,
        schedule: Schedule,
        action: &str,
        credential_ref: Option<&str>,
        first_fire_base_ms: u64,
    ) -> TaskId {
        let id = self.next_id;
        self.next_id += 1;
        let next_fire_ms = compute_initial_fire(schedule, first_fire_base_ms);
        self.tasks.push(Task {
            id, name: name.into(), schedule,
            action: action.into(),
            credential_ref: credential_ref.map(String::from),
            next_fire_ms, enabled: true,
        });
        id
    }

    pub fn remove_task(&mut self, id: TaskId) -> bool {
        match self.tasks.iter().position(|t| t.id == id) {
            Some(pos) => { self.tasks.remove(pos); true }
            None => false,
        }
    }

    pub fn set_enabled(&mut self, id: TaskId, enabled: bool) -> bool {
        if let Some(t) = self.tasks.iter_mut().find(|t| t.id == id) {
            t.enabled = enabled; true
        } else { false }
    }

    pub fn tasks(&self) -> &[Task] { &self.tasks }
    pub fn get_task(&self, id: TaskId) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == id)
    }
    pub fn history(&self) -> &[Run] { &self.history }
    pub fn history_for(&self, task_id: TaskId) -> Vec<&Run> {
        self.history.iter().filter(|r| r.task_id == task_id).collect()
    }

    /// Fire every task whose `next_fire_ms` ≤ `now_ms`. Returns the runs
    /// produced by this tick in the order they were fired. Dispatch is
    /// synchronous (tests use fakes; production uses real handlers).
    pub fn tick<F>(&mut self, now_ms: u64, mut dispatch: F) -> Vec<Run>
    where F: FnMut(DispatchContext) -> DispatchResult {
        // Take a snapshot of due task ids and the state we need.
        let due: Vec<TaskId> = self.tasks.iter()
            .filter(|t| t.enabled && t.next_fire_ms <= now_ms)
            .map(|t| t.id).collect();

        let mut new_runs = Vec::new();
        for id in due {
            // Borrow task immutably to compute dispatch; result used to schedule next.
            let (next_fire, disable, status, msg) = {
                let task = self.tasks.iter().find(|t| t.id == id).unwrap();
                let started = now_ms;
                let result = dispatch(DispatchContext { task, now_ms });
                let finished = now_ms;
                let (status, msg) = match result {
                    Ok(m) => (RunStatus::Succeeded, m),
                    Err(e) => (RunStatus::Failed, e),
                };
                let (nf, disable) = schedule_next(task.schedule, task.next_fire_ms, now_ms);
                let run = Run { task_id: id, started_ms: started, finished_ms: finished,
                                 status, message: msg.clone() };
                self.history.push(run.clone());
                new_runs.push(run);
                (nf, disable, status, msg)
            };
            let _ = (status, msg);
            if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
                task.next_fire_ms = next_fire;
                if disable { task.enabled = false; }
            }
        }
        new_runs
    }

    /// Returns a map of task_id → next_fire_ms for scheduling UIs.
    pub fn next_fires(&self) -> HashMap<TaskId, u64> {
        self.tasks.iter().map(|t| (t.id, t.next_fire_ms)).collect()
    }
}

impl Default for Scheduler {
    fn default() -> Self { Self::new() }
}

fn compute_initial_fire(schedule: Schedule, base_ms: u64) -> u64 {
    match schedule {
        Schedule::EveryMs(interval) => base_ms.saturating_add(interval),
        Schedule::AtMs(t) => t,
        Schedule::Daily { hour, minute } => {
            next_daily(base_ms, hour, minute)
        }
    }
}

/// Next wall-clock time at `hour:minute` strictly after `base_ms`.
/// Treats `base_ms` as ms-since-midnight-UTC cycles (no timezone, kept simple).
fn next_daily(base_ms: u64, hour: u8, minute: u8) -> u64 {
    const DAY_MS: u64 = 86_400_000;
    let day = base_ms / DAY_MS;
    let day_start = day * DAY_MS;
    let target_in_day = (hour as u64) * 3_600_000 + (minute as u64) * 60_000;
    let candidate = day_start + target_in_day;
    if candidate > base_ms { candidate } else { candidate + DAY_MS }
}

/// Compute next fire time and whether to disable the task after firing.
fn schedule_next(schedule: Schedule, prev_fire_ms: u64, now_ms: u64) -> (u64, bool) {
    match schedule {
        Schedule::EveryMs(interval) => {
            // Skip missed runs: advance to the next boundary strictly after now.
            let mut next = prev_fire_ms.saturating_add(interval);
            while next <= now_ms { next = next.saturating_add(interval); }
            (next, false)
        }
        Schedule::AtMs(_) => (u64::MAX, true),       // one-shot, disable
        Schedule::Daily { hour, minute } => {
            (next_daily(now_ms, hour, minute), false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn always_ok(_ctx: DispatchContext) -> DispatchResult {
        Ok("ok".into())
    }

    fn always_fail(_ctx: DispatchContext) -> DispatchResult {
        Err("simulated failure".into())
    }

    #[test]
    fn every_ms_schedule_fires_repeatedly() {
        let mut s = Scheduler::new();
        s.add_task("ping", Schedule::EveryMs(100), "ping", None, 0);

        let r1 = s.tick(100, always_ok);
        assert_eq!(r1.len(), 1);
        let r2 = s.tick(200, always_ok);
        assert_eq!(r2.len(), 1);
        let r3 = s.tick(250, always_ok);
        assert_eq!(r3.len(), 0, "no new fire yet");
        let r4 = s.tick(300, always_ok);
        assert_eq!(r4.len(), 1);
        assert_eq!(s.history().len(), 3);
    }

    #[test]
    fn tick_before_fire_time_produces_no_runs() {
        let mut s = Scheduler::new();
        s.add_task("later", Schedule::AtMs(1000), "noop", None, 0);
        assert!(s.tick(500, always_ok).is_empty());
        assert_eq!(s.history().len(), 0);
    }

    #[test]
    fn at_ms_schedule_is_one_shot() {
        let mut s = Scheduler::new();
        let id = s.add_task("once", Schedule::AtMs(1000), "do-it", None, 0);
        s.tick(1000, always_ok);
        assert_eq!(s.history().len(), 1);
        assert!(!s.get_task(id).unwrap().enabled);
        s.tick(5000, always_ok);
        assert_eq!(s.history().len(), 1, "disabled one-shot must not re-fire");
    }

    #[test]
    fn missed_runs_are_skipped_not_backfilled() {
        // Task scheduled every 100ms, then scheduler is offline for 5 seconds.
        let mut s = Scheduler::new();
        s.add_task("hourly", Schedule::EveryMs(100), "noop", None, 0);
        // Tick far ahead — simulates 5 seconds of downtime.
        let runs = s.tick(5000, always_ok);
        // Expect ONE run, not 50.
        assert_eq!(runs.len(), 1, "cron-like: skip missed, not backfill");
        // Next fire should be 100ms after 5000, not catching up from the past.
        let next = s.next_fires()[&1];
        assert!(next > 5000 && next <= 5100, "next fire should be 5100 ±, got {next}");
    }

    #[test]
    fn failed_runs_are_recorded_with_status() {
        let mut s = Scheduler::new();
        s.add_task("flaky", Schedule::EveryMs(100), "noop", None, 0);
        let runs = s.tick(100, always_fail);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].status, RunStatus::Failed);
        assert!(runs[0].message.contains("simulated failure"));
        // Failure doesn't disable the schedule.
        assert!(s.tasks()[0].enabled);
    }

    #[test]
    fn disabled_task_does_not_fire() {
        let mut s = Scheduler::new();
        let id = s.add_task("paused", Schedule::EveryMs(100), "noop", None, 0);
        s.set_enabled(id, false);
        let runs = s.tick(500, always_ok);
        assert!(runs.is_empty());
    }

    #[test]
    fn remove_task_removes() {
        let mut s = Scheduler::new();
        let id = s.add_task("doomed", Schedule::EveryMs(100), "noop", None, 0);
        assert!(s.remove_task(id));
        assert_eq!(s.tasks().len(), 0);
        assert!(!s.remove_task(id));
    }

    #[test]
    fn credential_ref_passed_to_dispatch() {
        let mut s = Scheduler::new();
        s.add_task("needs-creds", Schedule::AtMs(100), "login", Some("github"), 0);
        let mut seen: Option<String> = None;
        let capture = |ctx: DispatchContext| {
            seen = ctx.task.credential_ref.clone();
            Ok("done".into())
        };
        s.tick(100, capture);
        assert_eq!(seen, Some("github".into()));
    }

    #[test]
    fn daily_schedule_fires_at_time_of_day() {
        let mut s = Scheduler::new();
        // Base ms = 0 = midnight UTC. Task at 09:30 = 34_200_000 ms.
        s.add_task("morning", Schedule::Daily { hour: 9, minute: 30 },
                   "noop", None, 0);
        let next = s.next_fires()[&1];
        assert_eq!(next, 9 * 3_600_000 + 30 * 60_000);

        // At 09:29 — not yet.
        assert!(s.tick(34_199_999, always_ok).is_empty());
        // At 09:30 exactly — fires.
        assert_eq!(s.tick(34_200_000, always_ok).len(), 1);
        // Next fire is the following day.
        let next2 = s.next_fires()[&1];
        assert_eq!(next2, 34_200_000 + 86_400_000);
    }

    #[test]
    fn history_for_task_filters() {
        let mut s = Scheduler::new();
        let a = s.add_task("A", Schedule::EveryMs(100), "noop", None, 0);
        let _b = s.add_task("B", Schedule::EveryMs(100), "noop", None, 0);
        s.tick(100, always_ok);
        s.tick(200, always_ok);
        assert_eq!(s.history_for(a).len(), 2);
        assert_eq!(s.history().len(), 4);
    }
}
