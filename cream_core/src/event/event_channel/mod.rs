use std::sync::Arc;

use self::channel::RawChannel;

use super::{EventEmitter, EventReceiver};

mod channel;
pub mod data;
pub mod emitter;
pub mod getter;
mod header;
pub mod setter;

pub const GLOBAL_EVENT_RECEIVER_NAME: &str = "@global";

pub(crate) fn one_channel() -> (EventEmitter, EventReceiver) {
    let chan = Arc::new(RawChannel::new());
    (
        EventEmitter::new((), chan.clone()),
        EventReceiver::new(chan),
    )
}
