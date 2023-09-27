use irisia::{
    element::{Element, ElementUpdate, LayoutElements, PropsUpdateWith},
    event::standard::{ElementAbandoned, PointerDown, PointerEntered, PointerLeft, PointerOut},
    exit_app,
    primitive::{Pixel, Point},
    read_style,
    skia_safe::{Color, Color4f, Paint, Rect},
    style,
    style::StyleColor,
    ElModel, Event, Result, StaticWindowEvent, Style, StyleReader,
};
use tokio::select;

#[derive(Style, Clone, Copy, PartialEq)]
#[style(from)]
pub struct StyleWidth(Pixel);

#[derive(Style, Clone, Copy, PartialEq)]
#[style(from)]
pub struct StyleHeight(Pixel);

#[irisia::props(updater = "RectProps", watch)]
pub struct Rectangle {
    #[props(default = "false")]
    is_force: bool,

    #[props(default = "Color::CYAN")]
    force_color: Color,

    #[props(read_style(stdin))]
    style: RectStyles,
}

#[derive(StyleReader, PartialEq)]
struct RectStyles {
    width: Option<StyleWidth>,
    height: Option<StyleHeight>,
    color: Option<StyleColor>,
}

impl Element for Rectangle {
    type BlankProps = RectProps;

    fn render(
        &mut self,
        this: &ElModel!(),
        mut content: irisia::element::RenderElement,
    ) -> irisia::Result<()> {
        let region = this.draw_region();

        let end_point = Point(
            region.0 .0 + self.style.width.map(|x| x.0).unwrap_or(Pixel(50.0)),
            region.0 .1 + self.style.height.map(|h| h.0).unwrap_or(Pixel(50.0)),
        );

        this.set_interact_region(Some((region.0, end_point)));

        let rect = Rect::new(
            region.0 .0.to_physical(),
            region.0 .1.to_physical(),
            end_point.0.to_physical(),
            end_point.1.to_physical(),
        );

        let color = if self.is_force {
            self.force_color
        } else {
            self.style.color.unwrap_or(StyleColor(Color::GREEN)).0
        };

        let paint = Paint::new(Color4f::from(color), None);
        content.canvas().draw_rect(rect, &paint);
        Ok(())
    }
}

impl<Pr> ElementUpdate<Pr> for Rectangle
where
    Self: PropsUpdateWith<Pr>,
{
    fn el_create(this: &ElModel!(), props: Pr) -> Self {
        this.listen()
            .trusted()
            .asyn()
            .spawn(|_: PointerEntered, this| async move {
                this.el_write().await.unwrap().is_force = true;
            });

        this.listen()
            .trusted()
            .asyn()
            .spawn(|_: PointerOut, this| async move {
                this.el_write().await.unwrap().is_force = false;
            });

        Self::props_create_with(props)
    }

    fn el_update(&mut self, this: &ElModel!(), props: Pr, _: bool) -> bool {
        self.props_update_with(props).unchanged
    }
}

#[derive(Event, Clone)]
pub struct MyRequestClose;

pub struct Flex;

impl Element for Flex {
    type BlankProps = ();

    fn draw_region_changed(&mut self, this: &ElModel!(), _: irisia::primitive::Region) {
        flex_layout(this, this.layout_children().unwrap()).unwrap();
    }

    fn set_children(&self, this: &ElModel!()) {
        flex_layout(this, this.set_children(this.slot())).unwrap();
    }
}

impl ElementUpdate<()> for Flex {
    fn el_create(this: &ElModel!(), props: ()) -> Self {
        Flex
    }

    fn el_update(&mut self, this: &ElModel!(), props: (), equality_matters: bool) -> bool {
        true
    }
}

fn flex_layout(this: &ElModel!(Flex), layouter: LayoutElements) -> Result<()> {
    let (start, end) = this.draw_region();
    let abs = end - start;
    let len = layouter.len();
    let width = abs.0 / len as f32;

    let mut index = 0;
    layouter.layout(|()| {
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
