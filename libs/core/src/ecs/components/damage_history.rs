use amethyst::ecs::prelude::{Component, DenseVecStorage, FlaggedStorage};
use serde_derive::{Deserialize, Serialize};

pub struct DamageHistory {
    pub history: Vec<DamageHistoryEntries>,
}

impl Component for DamageHistory {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

impl DamageHistory {
    pub fn new(frame_number: u64) -> Self {
        Self {
            history: vec![DamageHistoryEntries::new(frame_number)],
        }
    }

    pub fn add_entry(&mut self, frame_number: u64, entry: DamageHistoryEntry) {
        log::trace!("Added damage entry (frame {}): {:?}", frame_number, entry);

        self.reserve_entries(frame_number);

        let damage_entries = self
            .history
            .iter_mut()
            .rev()
            .find(|entries| entries.frame_number == frame_number)
            .unwrap_or_else(|| {
                panic!(
                    "Expected reserved damage entries for frame {}",
                    frame_number
                )
            });

        damage_entries.entries.push(entry);
    }

    pub fn reset_entries(&mut self, frame_number: u64) {
        self.reserve_entries(frame_number);

        let damage_entries = self
            .history
            .iter_mut()
            .rev()
            .find(|entries| entries.frame_number == frame_number)
            .unwrap_or_else(|| {
                panic!(
                    "Expected reserved damage entries for frame {}",
                    frame_number
                )
            });
        damage_entries.entries.clear();
    }

    pub fn get_entries(&self, frame_number: u64) -> &DamageHistoryEntries {
        let i = frame_number
            - self
                .history
                .first()
                .expect("Expected at least one reserved entry")
                .frame_number;
        &self.history[i as usize]
    }

    fn reserve_entries(&mut self, frame_number: u64) {
        let start_frame_number = self
            .history
            .last()
            .map(|last_entries| last_entries.frame_number + 1)
            .expect("Expected at least one reserved entry");
        for added_frame_number in start_frame_number..=frame_number {
            self.history
                .push(DamageHistoryEntries::new(added_frame_number));
        }
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
