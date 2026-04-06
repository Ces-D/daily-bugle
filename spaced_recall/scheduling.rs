use anyhow::{Result, anyhow};
use chrono::{DateTime, TimeDelta, Utc};
use fsrs::{DEFAULT_PARAMETERS, FSRS, MemoryState, NextStates};

use crate::model::{ItemState, Rating};

const DEFAULT_DESIRED_RETENTION: f32 = 0.9;

pub struct Scheduler {
    fsrs: FSRS,
}

impl Scheduler {
    pub fn new() -> Result<Self> {
        let f = FSRS::new(Some(&DEFAULT_PARAMETERS))?;
        Ok(Self { fsrs: f })
    }

    pub fn preview(&self, item: &ItemState, now: DateTime<Utc>) -> Result<NextStates> {
        let memory = memory_state_of(item);
        let elapsed = days_elapsed(item, now);
        self.fsrs
            .next_states(memory, DEFAULT_DESIRED_RETENTION, elapsed)
            .map_err(|e| anyhow!("{e}"))
    }

    pub fn process_review(
        &self,
        item: &ItemState,
        rating: Rating,
        now: DateTime<Utc>,
    ) -> Result<(MemoryState, DateTime<Utc>)> {
        let next_states = self.preview(item, now)?;

        let chosen = match rating {
            Rating::Again => next_states.again,
            Rating::Hard => next_states.hard,
            Rating::Good => next_states.good,
            Rating::Easy => next_states.easy,
        };

        let interval_days = chosen.interval.round().max(1.0) as i64;
        let due_at = now
            + TimeDelta::try_days(interval_days)
                .ok_or_else(|| anyhow!("invalid interval: {interval_days}"))?;

        Ok((chosen.memory, due_at))
    }
}

fn memory_state_of(item: &ItemState) -> Option<MemoryState> {
    match (item.stability, item.difficulty) {
        (Some(s), Some(d)) => Some(MemoryState {
            stability: s as f32,
            difficulty: d as f32,
        }),
        _ => None,
    }
}

fn days_elapsed(item: &ItemState, now: DateTime<Utc>) -> u32 {
    item.last_reviewed_at
        .map(|reviewed| now.signed_duration_since(reviewed).num_days().max(0) as u32)
        .unwrap_or(0)
}
