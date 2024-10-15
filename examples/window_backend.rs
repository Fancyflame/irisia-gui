use irisia::{
    application::IncomingPointerEvent,
    el_model::{EMCreateCtx, ElInputWatcher, ElementAccess, LayerRebuilder},
    element::ElementInterfaces,
    event::standard::{PointerEntered, PointerOut},
    primitive::Point,
    skia_safe::{Color, Color4f, Paint, Rect},
    structure::ChildBox,
    user_props, Event, Result, WriteStyle,
};

pub struct Rectangle {
    is_force: bool,
    props: RectProps,
    access: ElementAccess,
}

#[user_props(name = "AAA")]
pub struct RectProps {
    #[props(optioned)]
    pub force_color: Color,

    #[props(default)]
    pub force_height: Option<f32>,
}

impl ElementInterfaces for Rectangle {
    type Props<'a> = RectProps;
    const REQUIRE_INDEPENDENT_LAYER: bool = false;

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

        access.set_interact_region(Some((Default::default(), (Point(50.0, 50.0)))));

        Self {
            is_force: false,
            props,
            access,
        }
    }

    fn spread_event(&mut self, _: &IncomingPointerEvent) -> bool {
        false
    }

    fn on_draw_region_changed(&mut self, _: irisia::primitive::Region) {}

    fn render(&mut self, lr: &mut LayerRebuilder, _: std::time::Duration) -> Result<()> {
        let region = self.access.draw_region();
        let styles = RectStyles::from_style(self.access.styles());

        let height = match *self.props.force_height.read() {
            Some(h) => h,
            None => styles
                .height
                .map(|h| h.0.to_resolved(&self.access))
                .unwrap_or(50.0),
        };

        let end_point = Point(
            region.0 .0
                + styles
                    .width
                    .map(|x| x.0.to_resolved(&self.access))
                    .unwrap_or(50.0),
            region.0 .1 + height,
        );

        self.access.set_interact_region(Some((region.0, end_point)));

        let rect = Rect::new(region.0 .0, region.0 .1, end_point.0, end_point.1);

        let color = if self.is_force {
            self.props
                .force_color
                .as_ref()
                .map(|color| *color.read())
                .unwrap_or(Color::BLACK)
        } else {
            styles.color.unwrap_or(sty::Color(Color::GREEN)).0
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
    type Props<'a> = ();

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

    fn spread_event(&mut self, ipe: &IncomingPointerEvent) -> bool {
        self.children.emit_event(ipe)
    }

    fn on_draw_region_changed(&mut self, _: irisia::primitive::Region) {}

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
