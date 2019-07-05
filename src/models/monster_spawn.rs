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

    /// Ah
    /// Gone a little far
    /// Gone a little far this time for somethin'
    /// How was I to know?
    /// How was I to know this high came rushing?
    ///
    /// These monsters will come def sooner than the new Tame Impala album.
    Borderline,
}
