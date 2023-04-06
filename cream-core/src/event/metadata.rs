#[derive(Debug, Clone, Copy)]
pub struct EventMetadata {
    pub(crate) is_system_event: bool,
}

impl EventMetadata {
    pub fn new() -> Self {
        EventMetadata {
            is_system_event: false,
        }
    }

    pub(crate) fn new_sys() -> Self {
        EventMetadata {
            is_system_event: true,
        }
    }

    pub fn is_system_event(&self) -> bool {
        self.is_system_event
    }
}

impl Default for EventMetadata {
    fn default() -> Self {
        Self::new()
    }
}
