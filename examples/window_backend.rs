use irisia::{
    application::IncomingPointerEvent,
    data_flow::Register,
    el_model::{EMCreateCtx, ElementAccess},
    element::{ElementInterfaces, Render},
    event::standard::{PointerEntered, PointerOut},
    primitive::{Point, Region},
    skia_safe::{Color, Color4f, Paint, Rect},
    structure::ChildBox,
    user_props, Event, Result,
};

pub struct Rectangle {
    is_force: Register<bool>,
    props: RectProps,
    access: ElementAccess,
}

#[user_props(name = "AAA")]
pub struct RectProps {
    #[props(optioned)]
    pub color: Color,

    #[props(default)]
    pub height: Option<f32>,
}

impl ElementInterfaces for Rectangle {
    type Props<'a> = RectProps;
    type SlotData = ();
    const REQUIRE_INDEPENDENT_LAYER: bool = false;

    fn create<Slt>(props: Self::Props<'_>, _: Slt, access: ElementAccess, _: &EMCreateCtx) -> Self
    where
        Slt: irisia::structure::StructureCreate<()>,
    {
        let is_force_reg = Register::new(false);

        let is_force = is_force_reg.clone();
        access.listen().trusted().spawn(move |_: PointerEntered| {
            println!("entered");
            is_force.set(true);
        });

        let is_force = is_force_reg.clone();
        access.listen().trusted().spawn(move |_: PointerOut| {
            is_force.set(false);
        });

        Self {
            is_force: Register::new(false),
            props,
            access,
        }
    }

    fn spread_event(&mut self, _: &IncomingPointerEvent) -> bool {
        false
    }

    fn on_draw_region_change(&mut self) {}

    fn render(&mut self, args: Render) -> Result<()> {
        let region = self.access.draw_region();

        let height = self.props.height.read().unwrap_or(50.0);
        let end_point = Point(region.left_top.0 + 80.0, region.left_top.1 + height);

        self.access.set_interact_region(Region {
            left_top: region.left_top,
            right_bottom: end_point,
        });

        let rect = Rect::new(
            region.left_top.0,
            region.left_top.1,
            end_point.0,
            end_point.1,
        );

        let mut color = Color::GREEN;
        if !*self.is_force.read() {
            if let Some(props_color) = self.props.color.as_ref().map(|color| *color.read()) {
                color = props_color;
            }
        }

        let paint = Paint::new(Color4f::from(color), None);
        args.canvas.draw_rect(rect, &paint);
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
    type SlotData = ();

    fn create<Slt>(_: Self::Props<'_>, slot: Slt, access: ElementAccess, ctx: &EMCreateCtx) -> Self
    where
        Slt: irisia::structure::StructureCreate<()>,
    {
        Self {
            children: ChildBox::new(slot, ctx),
            access,
        }
    }

    fn spread_event(&mut self, ipe: &IncomingPointerEvent) -> bool {
        self.children.emit_event(ipe)
    }

    fn on_draw_region_change(&mut self) {}

    fn render(&mut self, args: Render) -> Result<()> {
        self.flex_layout()?;
        self.children.render(args)
    }
}

impl Flex {
    fn flex_layout(&mut self) -> Result<()> {
        let Region {
            left_top: start,
            right_bottom: end,
        } = self.access.draw_region();

        let abs = end - start;
        let len = self.children.len();
        let width = abs.0 / len as f32;

        let mut index = 0;
        self.children.layout(|_| {
            if index >= len {
                return None;
            }

            let region = Region::new(
                Point(width * index as f32, start.1),
                Point(width * (index + 1) as f32, end.1),
            );
            index += 1;
            Some(region)
        })
    }
}
