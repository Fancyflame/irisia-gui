use std::ops::Range;

use irisia::primary::Point;
use irisia_core::{
    event::{
        element_handle::ElementHandle,
        standard::Blured,
        standard::{PointerDown, PointerMove, PointerUp},
        EventDispatcher,
    },
    skia_safe::Point as SkiaPoint,
};

use super::SelectionRange;

fn utf16_index_to_byte_index(s: &str, index: usize) -> Option<usize> {
    s.char_indices()
        .map(|ch| ch.0)
        .chain(std::iter::once(s.len()))
        .take(index + 1)
        .last()
}

impl super::TextBox {
    pub(super) async fn start_selection_runtime(
        win_ed: EventDispatcher,
        eh: ElementHandle,
        sel: SelectionRange,
    ) {
        loop {
            let pd = tokio::select! {
                pd = eh.recv_sys::<PointerDown>() => pd,
                _ = eh.recv_sys::<Blured>() => {
                    *sel.lock().unwrap() = None;
                    continue;
                }
            };

            eh.focus().await;

            if !pd.is_current {
                eh.recv_sys::<PointerUp>().await;
                continue;
            }

            let mut range = (pd.position, pd.position);
            *sel.lock().unwrap() = Some(range);

            loop {
                let pm = tokio::select! {
                    pm = win_ed.recv_sys::<PointerMove>() => pm,
                    _ = win_ed.recv_sys::<PointerUp>() => break
                };

                range.1 = pm.position;
                *sel.lock().unwrap() = Some(range);
            }
        }
    }

    pub(super) fn get_selection_range(&self, s: &str) -> Option<Range<usize>> {
        let r = (*self.selection_range.lock().unwrap())?;
        let paragraph = self.paragraph.as_ref()?;

        let get_word_b = |point: Point| {
            let pos = paragraph
                .get_glyph_position_at_coordinate(SkiaPoint::new(point.0 as _, point.1 as _));

            utf16_index_to_byte_index(s, pos.position as _)
        };
        let pos1 = get_word_b(r.0)?;
        let pos2 = get_word_b(r.1)?;

        Some(pos1.min(pos2)..pos1.max(pos2))
    }
}
