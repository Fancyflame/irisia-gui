use std::time::Duration;

use crate::event::standard::{
    Click, PointerDown, PointerEntered, PointerMove, PointerOut, PointerUp,
};

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

    pub async fn double_click(&self) {
        loop {
            self.recv_sys::<Click>().await;
            if tokio::time::timeout(Duration::from_millis(400), self.recv_sys::<Click>())
                .await
                .is_ok()
            {
                break;
            }
        }
    }

    pub async fn hold(&self) {
        loop {
            self.recv_sys::<PointerDown>().await;
            if tokio::time::timeout(Duration::from_secs(1), self.recv_sys::<PointerUp>())
                .await
                .is_err()
            {
                break;
            }
        }
    }
}
