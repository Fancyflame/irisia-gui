use std::time::Duration;

use crate::event::standard::{PointerDown, PointerEntered, PointerMove, PointerOut, PointerUp};

use super::EventDispatcher;

impl EventDispatcher {
    pub async fn hover(&self) {
        tokio::select! {
            _ = self.recv_sys::<PointerUp>() => {},
            _ = self.recv_sys::<PointerMove>() => {}
        }

        loop {
            tokio::select! {
                _ = self.recv_sys::<PointerDown>() => {
                    self.recv_sys::<PointerUp>().await;
                }
                _ = self.recv_sys::<PointerOut>() => {
                    self.recv_sys::<PointerEntered>().await;
                }
                _ = tokio::time::sleep(Duration::from_secs(1)) => {
                    break;
                }
            }
        }
    }

    pub async fn hover_canceled(&self) {
        tokio::select! {
            _ = self.recv_sys::<PointerDown>() => {},
            _ = self.recv_sys::<PointerOut>() => {}
        }
    }
}
