use irisia::{
    application::IncomingPointerEvent,
    coerce_signal,
    el_model::{EMCreateCtx, ElementAccess},
    element::{children_utils::ChildrenUtils, ComponentTemplate, ElementInterfaces, Render},
    event::standard::{PointerEntered, PointerOut},
    hook::{Effect, Provider, Signal},
    model::{iter::VisitModel, reactive::reactive},
    primitive::{Point, Region},
    skia_safe::{Color, Color4f, Paint, Rect},
    Event, Result,
};

pub struct Rectangle {
    is_force: Signal<bool>,
    props: RectProps,
    access: ElementAccess,
}

#[derive(Clone)]
#[irisia::props]
pub struct RectProps {
    #[props(default)]
    pub color: Option<Color>,
    pub height: Option<f32>,
}

impl ElementInterfaces for Rectangle {
    type Props = RectProps;
    type ChildMapper = ();
    const REQUIRE_INDEPENDENT_LAYER: bool = false;

    fn create<Slt>(
        props: &Self::Props,
        access: ElementAccess,
        _: irisia::hook::ProviderObject<Slt>,
        _: &EMCreateCtx,
    ) -> Self
    where
        Slt: irisia::model::DesiredVModel<Self::ChildMapper> + 'static,
    {
        let is_force_sig = Signal::state(false);

        let is_force = is_force_sig.clone();
        access.listen().trusted().spawn(move |_: PointerEntered| {
            println!("entered");
            is_force.set(true);
        });

        let is_force = is_force_sig.clone();
        access.listen().trusted().spawn(move |_: PointerOut| {
            is_force.set(false);
        });

        access.redraw_when((
            is_force_sig.clone(),
            props.color.clone(),
            props.height.clone(),
        ));

        Self {
            is_force: is_force_sig,
            props: props.clone(),
            access,
        }
    }

    fn spread_event(&mut self, _: &IncomingPointerEvent) -> bool {
        false
    }

    fn on_draw_region_change(&mut self) {}

    fn render(&mut self, args: Render) -> Result<()> {
        let region = self.access.draw_region().unwrap_or_default();

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
            if let Some(props_color) = self.props.color.read().as_ref().cloned() {
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
    children: Signal<dyn VisitModel<()>>,
    access: ElementAccess,
}

impl ElementInterfaces for Flex {
    type Props = ();
    type ChildMapper = ();

    fn create<Slt>(
        props: &Self::Props,
        access: ElementAccess,
        slot: irisia::hook::ProviderObject<Slt>,
        ctx: &EMCreateCtx,
    ) -> Self
    where
        Slt: irisia::model::DesiredVModel<Self::ChildMapper> + 'static,
    {
        access.redraw_when(slot.clone());
        let r = reactive(
            |packer, slot| {
                packer.apply_model(slot);
            },
            ctx,
            slot,
        );
        Flex {
            children: coerce_signal!(r),
            access,
        }
    }

    fn spread_event(&mut self, ipe: &IncomingPointerEvent) -> bool {
        self.children.write().emit_event(ipe)
    }

    fn on_draw_region_change(&mut self) {}

    fn render(&mut self, args: Render) -> Result<()> {
        self.flex_layout()?;
        self.children.write().render(args)
    }
}

impl Flex {
    fn flex_layout(&mut self) -> Result<()> {
        let Region {
            left_top: start,
            right_bottom: end,
        } = self.access.draw_region().unwrap_or_default();

        let abs = end - start;
        let len = self.children.read().compute_len();
        let width = abs.0 / len as f32;

        let mut index = 0;
        self.children.write().layout(|_| {
            if index >= len {
                return None;
            }

            let region = Region::new(
                Point(width * index as f32, start.1),
                Point(width * (index + 1) as f32, end.1),
            );
            index += 1;
            Some(region)
        });
        Ok(())
    }
}
