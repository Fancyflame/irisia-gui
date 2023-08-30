use super::styles::*;
use irisia::{
    element::{Element, UpdateElement},
    primitive::Region,
    skia_safe::{
        font_style::Width,
        textlayout::{FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, TextStyle},
        Color, FontMgr, FontStyle, Point as SkiaPoint,
    },
    skia_safe::{Color4f, ColorSpace, Paint},
    style::{StyleColor, StyleContainer},
    StyleReader, UpdateWith,
};

use crate::box_styles::BoxStyleRenderer;

use super::{selection::SelectionRtMgr, OwnedProps, TextBox};

impl Element for TextBox {
    type BlankProps = super::Props;
}

impl<Pr> UpdateWith<UpdateElement<'_, Self, Pr>> for TextBox
where
    OwnedProps: UpdateWith<Pr>,
{
    fn create_with(updater: UpdateElement<'_, Self, Pr>) -> Self {
        let mut font_collection = FontCollection::new();
        font_collection.set_default_font_manager(FontMgr::new(), None);
        TextBox {
            props: OwnedProps::create_with(updater.props),
            font_collection,
            paragraph: None,
            selection: None,
            selection_rt_mgr: SelectionRtMgr::new((*updater.handle).clone()),
        }
    }
}

impl Element<EmptyStructure> for TextBox {
    type Props<'a> = Props<'a>;

    fn render<'a>(
        &mut self,
        Frame {
            props,
            children,
            content,
        }: Frame<Self, EmptyStructure>,
    ) -> irisia_core::Result<()> {
        let drawing_region =
            BoxStyleRenderer::draw_border_limited(styles, content.canvas(), drawing_region);

        let styles = TextBoxStyles::read_style(styles);
        let selection = self.selection_rt_mgr.get_selection_range(
            drawing_region.0,
            &self.paragraph,
            &props.text,
        );

        if let (Some(p), false) = (
            &self.paragraph,
            self.needs_redraw(&styles, &props, drawing_region, &selection),
        ) {
            p.paint(
                content.canvas(),
                SkiaPoint::new(drawing_region.0 .0, drawing_region.0 .1),
            );
            return Ok(());
        }

        self.handle_user_select(props.user_select);

        let para_style = {
            let mut ps = ParagraphStyle::new();
            ps.set_height(drawing_region.1 .1 - drawing_region.0 .1);
            ps
        };

        let mut paragraph = self.render_paragraph(
            selection.clone(),
            ParagraphBuilder::new(&para_style, &self.font_collection),
            props.text,
            &parse_text_style(&styles),
        );

        paragraph.layout(drawing_region.1 .0 - drawing_region.0 .0);
        paragraph.paint(
            content.canvas(),
            SkiaPoint::new(drawing_region.0 .0, drawing_region.0 .1),
        );
        self.paragraph = Some(paragraph);

        self.update_previous_state(styles, props, drawing_region, selection);

        Ok(())
    }
}

impl TextBox {
    fn handle_user_select(&mut self, new_state: bool) {
        match (&self.previous_state, new_state) {
            (
                Some(OwnedState {
                    user_select: false, ..
                })
                | None,
                true,
            ) => {
                self.selection_rt_mgr.start_runtime();
            }

            (
                Some(OwnedState {
                    user_select: true, ..
                })
                | None,
                false,
            ) => self.selection_rt_mgr.stop_runtime(),

            _ => {}
        }
    }

    fn render_paragraph(
        &self,
        selection: Option<Range<usize>>,
        mut paragraph_builder: ParagraphBuilder,
        text: &str,
        text_style: &TextStyle,
    ) -> Paragraph {
        match selection {
            Some(range) => {
                let (start, end) = (range.start, range.end);
                let mut selection_style = text_style.clone();
                to_selection_style(&mut selection_style);

                paragraph_builder
                    .push_style(&text_style)
                    .add_text(&text[..start])
                    .push_style(&selection_style)
                    .add_text(&text[range])
                    .push_style(&text_style)
                    .add_text(&text[end..]);
            }
            None => {
                paragraph_builder.push_style(&text_style).add_text(text);
            }
        }
        paragraph_builder.build()
    }

    fn needs_redraw(
        &self,
        styles: &TextBoxStyles,
        props: &Props,
        region: Region,
        selection: &Option<Range<usize>>,
    ) -> bool {
        let Some(prv) = &self.previous_state
        else {
            return true
        };

        prv.styles != *styles
            || prv.text != props.text
            || prv.user_select != props.user_select
            || prv.drawing_region != region
            || self.selection != *selection
    }

    fn update_previous_state(
        &mut self,
        styles: TextBoxStyles,
        props: Props,
        region: Region,
        selection: Option<Range<usize>>,
    ) {
        let owned_string = match self.previous_state.take() {
            Some(state) => state.text,
            None => props.text.into(),
        };

        self.previous_state = Some(OwnedState {
            text: owned_string,
            styles,
            drawing_region: region,
            user_select: props.user_select,
        });

        self.selection = selection;
    }
}

fn parse_text_style(style: &TextBoxStyles) -> TextStyle {
    let mut text_style = TextStyle::new();
    text_style
        .set_font_style(FontStyle::new(style.weight.0, Width::NORMAL, style.slant.0))
        .set_font_size(style.font_size.0.to_physical())
        .set_color(match &style.color {
            Some(c) => c.0,
            None => Color::BLACK,
        });
    text_style
}

fn to_selection_style(style: &mut TextStyle) {
    style.set_color(Color::WHITE);
    style.set_background_color(&Paint::new(
        Color4f::from(Color::from_argb(0xee, 0x4d, 0x90, 0xfe)),
        &ColorSpace::new_srgb(),
    ));
}
