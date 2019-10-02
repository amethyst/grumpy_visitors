use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use serde_derive::{Deserialize, Serialize};

use std::ops::Range;

use crate::{math::Vector2, net::NetIdentifier};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnActions(pub Vec<SpawnAction>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnAction {
    pub spawn_type: SpawnType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpawnType {
    /// Contains a position of the monster to spawn.
    Single {
        entity_net_id: Option<NetIdentifier>,
        position: Vector2,
    },

    /// Ah
    /// Gone a little far
    /// Gone a little far this time for somethin'
    /// How was I to know?
    /// How was I to know this high came rushing?
    ///
    /// These monsters will come def sooner than the new Tame Impala album.
    Borderline {
        count: u8,
        entity_net_id_range: Option<Range<NetIdentifier>>,
        side: Side,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Side {
    Top,
    Right,
    Bottom,
    Left,
}

impl Distribution<Side> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Side {
        match rng.gen_range(0, 4) {
            0 => Side::Top,
            1 => Side::Right,
            2 => Side::Bottom,
            _ => Side::Left,
        }
    }
}
