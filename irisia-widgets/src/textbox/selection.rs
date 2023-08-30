use std::{
    ops::Range,
    sync::{Arc, Mutex as SyncMutex},
};

use irisia::{
    element::ElementHandle,
    event::standard::{Blured, PointerDown, PointerEntered, PointerMove, PointerOut, PointerUp},
    primitive::{Pixel, Point},
    skia_safe::{textlayout::Paragraph, Point as SkiaPoint},
    winit::window::CursorIcon,
    WinitWindow,
};

use tokio::{sync::Mutex, task::JoinHandle};

pub(super) struct SelectionRtMgr {
    eh: Arc<ElementHandle>,
    sel: Arc<SyncMutex<Selection>>,
    handle: Option<JoinHandle<Option<()>>>,
}

#[derive(Default)]
struct Selection {
    cursor: Option<(Point, Point)>,
}

impl SelectionRtMgr {
    pub fn new(eh: Arc<ElementHandle>) -> Self {
        Self {
            eh,
            sel: Default::default(),
            handle: None,
        }
    }

    pub fn start_runtime(&mut self) {
        self.handle = Some(self.eh.daemon(start(self.eh.clone(), self.sel.clone())));
    }

    pub fn stop_runtime(&mut self) {
        if let Some(h) = self.handle.take() {
            h.abort();
        }
        self.sel.lock().unwrap().cursor = None;
    }

    pub fn get_selection_range(
        &self,
        cursor_offset: Point,
        paragraph: &Option<Paragraph>,
        s: &str,
    ) -> Option<Range<usize>> {
        let two_point = {
            let (start, end) = self.sel.lock().unwrap().cursor?;
            let checked_sub = |point: Point| {
                Point(
                    (point.0 - cursor_offset.0).max(Pixel(0.0)),
                    (point.1 - cursor_offset.1).max(Pixel(0.0)),
                )
            };
            (checked_sub(start), checked_sub(end))
        };
        let paragraph = paragraph.as_ref()?;

        let get_word_b = |Point(x, y): Point| {
            let pos = paragraph
                .get_glyph_position_at_coordinate(SkiaPoint::new(x.to_physical(), y.to_physical()));

            glyph_index_to_byte_index(s, pos.position as _)
        };
        let pos1 = get_word_b(two_point.0)?;
        let pos2 = get_word_b(two_point.1)?;

        Some(pos1.min(pos2)..pos1.max(pos2))
    }
}

fn glyph_index_to_byte_index(s: &str, index: usize) -> Option<usize> {
    s.char_indices()
        .map(|ch| ch.0)
        .chain(std::iter::once(s.len()))
        .take(index + 1)
        .last()
}

struct CursorIconSetter {
    showing_text_cursor: bool,
    text_selecting: bool,
    cursor_entered: bool,
}

impl CursorIconSetter {
    fn refresh(&mut self, win: &WinitWindow) {
        match (
            self.showing_text_cursor,
            self.text_selecting || self.cursor_entered,
        ) {
            (false, true) => {
                win.set_cursor_icon(CursorIcon::Text);
                self.showing_text_cursor = true;
            }
            (true, false) => {
                win.set_cursor_icon(CursorIcon::Default);
                self.showing_text_cursor = false;
            }
            _ => {}
        }
    }

    fn set_text_selecting(&mut self, win: &WinitWindow, value: bool) {
        self.text_selecting = value;
        self.refresh(win);
    }

    fn set_cursor_entered(&mut self, win: &WinitWindow, value: bool) {
        self.cursor_entered = value;
        self.refresh(win);
    }
}

async fn start(eh: Arc<ElementHandle>, sel: Arc<SyncMutex<Selection>>) {
    let cursor_icon_setter = Mutex::new(CursorIconSetter {
        showing_text_cursor: false,
        text_selecting: false,
        cursor_entered: false,
    });

    let ed = eh.event_dispatcher();
    let global_ed = eh.global().event_dispatcher();

    let a = async {
        loop {
            cursor_icon_setter
                .lock()
                .await
                .set_text_selecting(&eh.window(), false);

            let pd = tokio::select! {
                pd = ed.recv_sys::<PointerDown>() => pd,
                _ = ed.recv_sys::<Blured>() => {
                    sel.lock().unwrap().cursor = None;
                    continue;
                }
            };

            if !pd.is_current {
                ed.recv_sys::<PointerUp>().await;
                continue;
            }

            eh.focus();
            cursor_icon_setter
                .lock()
                .await
                .set_text_selecting(eh.window(), true);

            let mut range = (pd.position, pd.position);
            sel.lock().unwrap().cursor = Some(range);

            loop {
                let pm = tokio::select! {
                    pm = global_ed.recv_sys::<PointerMove>() => pm,
                    _ = global_ed.recv_sys::<PointerUp>() => break
                };

                range.1 = pm.position;
                sel.lock().unwrap().cursor = Some(range);
            }
        }
    };

    let b = async {
        ed.recv_sys::<PointerMove>().await;

        loop {
            tokio::select! {
                _ = ed.recv_sys::<PointerEntered>() => {
                    cursor_icon_setter
                        .lock()
                        .await
                        .set_cursor_entered(eh.window(), true);
                }

                _ = ed.recv_sys::<PointerOut>() => {
                    cursor_icon_setter
                        .lock()
                        .await
                        .set_cursor_entered(eh.window(), false);
                }
            }
        }
    };

    tokio::join!(a, b);
}
