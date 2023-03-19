#[derive(Debug, Clone, Copy)]
pub struct EventFlow {
    pub(crate) bubble: bool,
    pub(crate) is_current: bool,
}

impl EventFlow {
    pub fn bubble(&self) -> bool {
        self.bubble
    }

    pub fn is_current(&self) -> bool {
        self.is_current
    }
}
