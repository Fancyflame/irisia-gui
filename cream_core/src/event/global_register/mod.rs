use crate::primary::Region;

use super::event_state::wrap::WrappedEvents;

pub(crate) mod system_event_register;

pub trait GlobalEventRegister {
    fn listen_list(&mut self, callback: WrappedEvents, region: Region);
}
