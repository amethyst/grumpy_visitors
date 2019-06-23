pub struct SpawnActions(pub Vec<SpawnAction>);

pub struct SpawnAction {
    pub monsters: Count<String>,
    pub spawn_type: SpawnType,
}

pub struct Count<T> {
    pub entity: T,
    pub num: u8,
}

pub enum SpawnType {
    Random,
    Borderline,
}
