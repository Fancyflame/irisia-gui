#[derive(Debug, Clone, Copy)]
pub struct EventMetadata {
    pub(crate) is_trusted_event: bool,
}

impl EventMetadata {
    pub fn new() -> Self {
        EventMetadata {
            is_trusted_event: false,
        }
    }

    pub(crate) fn new_trusted() -> Self {
        EventMetadata {
            is_trusted_event: true,
        }
    }

    pub fn is_trusted_event(&self) -> bool {
        self.is_trusted_event
    }
}

impl Default for EventMetadata {
    fn default() -> Self {
        Self::new()
    }
}
