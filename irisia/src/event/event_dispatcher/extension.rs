use std::time::Duration;

use crate::event::standard::{
    Click, PointerDown, PointerEntered, PointerMove, PointerOut, PointerUp,
};

use super::EventDispatcher;

impl EventDispatcher {
    pub async fn hover(&self) {
        tokio::select! {
            _ = self.recv_trusted::<PointerUp>() => {},
            _ = self.recv_trusted::<PointerMove>() => {}
        }

        loop {
            tokio::select! {
                _ = self.recv_trusted::<PointerDown>() => {
                    self.recv_trusted::<PointerUp>().await;
                }
                _ = self.recv_trusted::<PointerOut>() => {
                    self.recv_trusted::<PointerEntered>().await;
                }
                _ = tokio::time::sleep(Duration::from_secs(1)) => {
                    break;
                }
            }
        }
    }

    pub async fn hover_canceled(&self) {
        tokio::select! {
            _ = self.recv_trusted::<PointerDown>() => {},
            _ = self.recv_trusted::<PointerOut>() => {}
        }
    }

    pub async fn double_click(&self) {
        loop {
            self.recv_trusted::<Click>().await;
            if tokio::time::timeout(Duration::from_millis(400), self.recv_trusted::<Click>())
                .await
                .is_ok()
            {
                break;
            }
        }
    }

    pub async fn hold(&self) {
        loop {
            self.recv_trusted::<PointerDown>().await;
            if tokio::time::timeout(Duration::from_secs(1), self.recv_trusted::<PointerUp>())
                .await
                .is_err()
            {
                break;
            }
        }
    }
}
