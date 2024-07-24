use std::{rc::Rc, time::Duration};

use irisia::{
    application::IncomingPointerEvent,
    data_flow::{register::Register, wire, ReadWire},
    el_model::{EMCreateCtx, ElInputWatcher, ElementAccess, LayerRebuilder},
    element::{ElementInterfaces, EmptyProps, FieldMustInit},
    event::standard::{ElementAbandoned, PointerDown, PointerEntered, PointerLeft, PointerOut},
    exit_app,
    primitive::{Pixel, Point},
    skia_safe::{Color, Color4f, Paint, Rect},
    structure::ChildBox,
    style::WriteStyle,
    Event, Result, Style, StyleReader, UserProps, WriteStyle,
};

#[derive(Style, Clone, Copy, PartialEq)]
pub struct StyleColor(Color);

#[derive(Style, Clone, Copy, PartialEq)]
#[style(from)]
pub struct StyleWidth(Pixel);

#[derive(Style, Clone, Copy, PartialEq)]
#[style(from)]
pub struct StyleHeight(Pixel);

pub struct Rectangle {
    is_force: bool,
    force_color: ReadWire<Color>,
    styles: ReadWire<RectStyles>,
    access: ElementAccess,
}

#[derive(UserProps)]
pub struct RectProps {
    #[props(required)]
    pub force_color: ReadWire<Color>,
}

#[derive(Default, WriteStyle)]
struct RectStyles {
    width: Option<StyleWidth>,
    height: Option<StyleHeight>,
    color: Option<StyleColor>,
}

impl ElementInterfaces for Rectangle {
    type Props<'a> = <RectProps as UserProps>::Props;

    fn create<Slt>(
        props: Self::Props<'_>,
        _: Slt,
        access: ElementAccess,
        watch_input: ElInputWatcher<Self>,
        _: &EMCreateCtx,
    ) -> Self
    where
        Slt: irisia::structure::StructureCreate,
    {
        let styles = {
            let access = access.clone();
            wire(move || RectStyles::from_style(access.styles()))
        };

        let wi = watch_input.clone();
        let access_cloned = access.clone();
        access.listen().trusted().spawn(move |_: PointerEntered| {
            println!("entered");
            wi.invoke_mut(|el| el.is_force = true);
            access_cloned.request_redraw();
        });

        let wi = watch_input.clone();
        let access_cloned = access.clone();
        access.listen().trusted().spawn(move |_: PointerOut| {
            wi.invoke_mut(|el| el.is_force = false);
            access_cloned.request_redraw();
        });

        access.set_interact_region(Some((
            Default::default(),
            (Point(Pixel(50.0), Pixel(50.0))),
        )));

        Self {
            is_force: false,
            force_color: RectProps::from(props).force_color,
            styles,
            access,
        }
    }

    fn children_emit_event(&mut self, _: &IncomingPointerEvent) -> bool {
        false
    }

    fn set_draw_region(&mut self, _: irisia::primitive::Region) {}

    fn render(&mut self, lr: &mut LayerRebuilder, _: std::time::Duration) -> Result<()> {
        let region = self.access.draw_region();
        let styles = self.styles.read();

        let end_point = Point(
            region.0 .0 + styles.width.map(|x| x.0).unwrap_or(Pixel(50.0)),
            region.0 .1 + styles.height.map(|h| h.0).unwrap_or(Pixel(50.0)),
        );

        self.access.set_interact_region(Some((region.0, end_point)));

        let rect = Rect::new(
            region.0 .0.to_physical(),
            region.0 .1.to_physical(),
            end_point.0.to_physical(),
            end_point.1.to_physical(),
        );

        let color = if self.is_force {
            *self.force_color.read()
        } else {
            self.styles
                .read()
                .color
                .unwrap_or(StyleColor(Color::GREEN))
                .0
        };

        let paint = Paint::new(Color4f::from(color), None);
        lr.canvas().draw_rect(rect, &paint);
        Ok(())
    }
}

#[derive(Event, Clone)]
pub struct MyRequestClose;

pub struct Flex {
    children: ChildBox,
    access: ElementAccess,
}

impl ElementInterfaces for Flex {
    type Props<'a> = EmptyProps;

    fn create<Slt>(
        _: Self::Props<'_>,
        slot: Slt,
        access: ElementAccess,
        _: ElInputWatcher<Self>,
        ctx: &EMCreateCtx,
    ) -> Self
    where
        Slt: irisia::structure::StructureCreate,
    {
        Self {
            children: ChildBox::new(slot, ctx),
            access,
        }
    }

    fn children_emit_event(&mut self, ipe: &IncomingPointerEvent) -> bool {
        self.children.emit_event(ipe)
    }

    fn set_draw_region(&mut self, _: irisia::primitive::Region) {}

    fn render(&mut self, lr: &mut LayerRebuilder, interval: std::time::Duration) -> Result<()> {
        self.flex_layout()?;
        self.children.render(lr, interval)
    }
}

impl Flex {
    fn flex_layout(&mut self) -> Result<()> {
        let (start, end) = self.access.draw_region();
        let abs = end - start;
        let len = self.children.len();
        let width = abs.0 / len as f32;

        let mut index = 0;
        self.children.layout(|_| {
            if index >= len {
                return None;
            }

            let region = (
                Point(width * index as f32, start.1),
                Point(width * (index + 1) as f32, end.1),
            );
            index += 1;
            Some(region)
        })
    }
}
