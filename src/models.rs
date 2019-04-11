use crate::data_resources::EntityGraphics;

pub struct SpawnActions(pub Vec<SpawnAction>);

pub struct SpawnAction {
    pub monsters: Count<String>,
}

pub struct Count<T> {
    pub entity: T,
    pub num: u8,
}

#[derive(Clone)]
pub struct MonsterDefinition {
    pub name: String,
    pub base_health: f32,
    pub base_speed: f32,
    pub base_attack: f32,
    pub graphics: EntityGraphics,
}
