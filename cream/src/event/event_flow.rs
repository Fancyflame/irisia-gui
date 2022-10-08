use getset::CopyGetters;

#[derive(Debug, Clone, CopyGetters)]
pub struct EventFlow {
    #[getset(get_copy = "pub")]
    pub(crate) bubble: bool,

    #[getset(get_copy = "pub")]
    pub(crate) is_exact: bool,
}

impl EventFlow {
    pub fn cancel_bubble(&mut self) {
        self.bubble = false;
    }
}
