use amethyst::ecs::prelude::{Component, DenseVecStorage, FlaggedStorage};
use serde_derive::{Deserialize, Serialize};

use std::cmp::Ordering;

#[derive(Default)]
pub struct DamageHistory {
    history: Vec<DamageHistoryEntries>,
}

impl Component for DamageHistory {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

impl DamageHistory {
    pub fn add_entry(&mut self, frame_number: u64, entry: DamageHistoryEntry) {
        let last_entries = &mut self.history.last_mut();
        if let Some(last_entries) = last_entries {
            match last_entries.frame_number.cmp(&frame_number) {
                Ordering::Greater => panic!(
                    "Adding timed out entries is not supported (at least not before multiplayer)"
                ),
                Ordering::Equal => {
                    last_entries.entries.push(entry);
                }
                _ => {}
            }
        } else {
            let mut last_entries = DamageHistoryEntries::new(frame_number);
            last_entries.entries.push(entry);
            self.history.push(last_entries);
        }
    }

    pub fn last_entries(&self) -> &DamageHistoryEntries {
        self.history.last().expect("Expected filled DamageHistory")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageHistoryEntries {
    pub frame_number: u64,
    pub entries: Vec<DamageHistoryEntry>,
}

impl DamageHistoryEntries {
    pub fn new(frame_number: u64) -> Self {
        Self {
            frame_number,
            entries: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageHistoryEntry {
    pub damage: f32,
}
