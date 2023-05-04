use std::{
    ops::Range,
    sync::{Arc, Mutex as SyncMutex},
};

use irisia::{
    event::standard::{PointerEntered, PointerOut},
    primary::Point,
    skia_safe::textlayout::Paragraph,
    winit::window::CursorIcon,
    WinitWindow,
};
use irisia_core::{
    element::ElementHandle,
    event::{
        standard::Blured,
        standard::{PointerDown, PointerMove, PointerUp},
        EventDispatcher,
    },
    skia_safe::Point as SkiaPoint,
};
use tokio::{sync::Mutex, task::JoinHandle};

pub(super) struct SelectionRtMgr {
    window: Arc<WinitWindow>,
    win_ed: EventDispatcher,
    eh: ElementHandle,
    sel: Arc<SyncMutex<Selection>>,
    handle: Option<JoinHandle<Option<()>>>,
}

#[derive(Default)]
struct Selection {
    cursor: Option<(Point, Point)>,
}

impl SelectionRtMgr {
    pub fn new(window: Arc<WinitWindow>, win_ed: EventDispatcher, eh: ElementHandle) -> Self {
        Self {
            window,
            win_ed,
            eh,
            sel: Default::default(),
            handle: None,
        }
    }

    pub fn start_runtime(&mut self) {
        self.handle = Some(self.eh.spawn(start(
            self.eh.clone(),
            self.sel.clone(),
            self.window.clone(),
            self.win_ed.clone(),
        )));
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
                    point.0.checked_sub(cursor_offset.0).unwrap_or_default(),
                    point.1.checked_sub(cursor_offset.1).unwrap_or_default(),
                )
            };
            (checked_sub(start), checked_sub(end))
        };
        let paragraph = paragraph.as_ref()?;

        let get_word_b = |point: Point| {
            let pos = paragraph
                .get_glyph_position_at_coordinate(SkiaPoint::new(point.0 as _, point.1 as _));

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

async fn start(
    eh: ElementHandle,
    sel: Arc<SyncMutex<Selection>>,
    win: Arc<WinitWindow>,
    win_ed: EventDispatcher,
) {
    let cursor_icon_setter = Mutex::new(CursorIconSetter {
        showing_text_cursor: false,
        text_selecting: false,
        cursor_entered: false,
    });

    let a = async {
        loop {
            cursor_icon_setter
                .lock()
                .await
                .set_text_selecting(&win, false);

            let pd = tokio::select! {
                pd = eh.recv_sys::<PointerDown>() => pd,
                _ = eh.recv_sys::<Blured>() => {
                    sel.lock().unwrap().cursor = None;
                    continue;
                }
            };

            if !pd.is_current {
                eh.recv_sys::<PointerUp>().await;
                continue;
            }

            eh.focus().await;
            cursor_icon_setter
                .lock()
                .await
                .set_text_selecting(&win, true);

            let mut range = (pd.position, pd.position);
            sel.lock().unwrap().cursor = Some(range);

            loop {
                let pm = tokio::select! {
                    pm = win_ed.recv_sys::<PointerMove>() => pm,
                    _ = win_ed.recv_sys::<PointerUp>() => break
                };

                range.1 = pm.position;
                sel.lock().unwrap().cursor = Some(range);
            }
        }
    };

    let b = async {
        eh.recv_sys::<PointerMove>().await;

        loop {
            tokio::select! {
                _ = eh.recv_sys::<PointerEntered>() => {
                    cursor_icon_setter
                        .lock()
                        .await
                        .set_cursor_entered(&win, true);
                }

                _ = eh.recv_sys::<PointerOut>() => {
                    cursor_icon_setter
                        .lock()
                        .await
                        .set_cursor_entered(&win, false);
                }
            }
        }
    };

    tokio::join!(a, b);
}
