use crate::global::pool::Pool;

pub(crate) mod pool;

pub struct Runtime {
    entities: Pool,
}

pub struct Entity {
    children: Vec,
}
