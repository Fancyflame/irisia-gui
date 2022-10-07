use getset::CopyGetters;

#[derive(Debug, Clone, CopyGetters)]
pub struct EventFlow {
    #[getset(get_copy = "pub")]
    pub(super) bubble: bool,

    #[getset(get_copy = "pub")]
    pub(super) is_exact: bool,
}

impl EventFlow {
    pub fn cancel_bubble(&mut self) {
        self.bubble = false;
    }
}
