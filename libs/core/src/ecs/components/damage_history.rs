use amethyst::ecs::prelude::{Component, DenseVecStorage, FlaggedStorage};

use std::time::Duration;

#[derive(Default)]
pub struct DamageHistory {
    history: Vec<DamageHistoryEntries>,
}

impl Component for DamageHistory {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

impl DamageHistory {
    pub fn add_entry(&mut self, time: Duration, entry: DamageHistoryEntry) {
        let last_entries = &mut self.history.last_mut();
        if let Some(last_entries) = last_entries {
            if last_entries.time > time {
                panic!(
                    "Adding timed out entries is not supported (at least not before multiplayer)"
                )
            } else if last_entries.time == time {
                last_entries.entries.push(entry);
            }
        } else {
            let mut last_entries = DamageHistoryEntries::new(time);
            last_entries.entries.push(entry);
            self.history.push(last_entries);
        }
    }

    pub fn last_entries(&self) -> &DamageHistoryEntries {
        self.history.last().expect("Expected filled DamageHistory")
    }
}

pub struct DamageHistoryEntries {
    pub time: Duration,
    pub entries: Vec<DamageHistoryEntry>,
}

impl DamageHistoryEntries {
    pub fn new(time: Duration) -> Self {
        Self {
            time,
            entries: Vec::new(),
        }
    }
}

pub struct DamageHistoryEntry {
    pub damage: f32,
}
