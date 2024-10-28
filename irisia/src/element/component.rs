use std::marker::PhantomData;

use crate::{
    application::event_comp::IncomingPointerEvent,
    el_model::{EMCreateCtx, ElementAccess, ElementModel},
    structure::{ChildBox, StructureCreate},
    ElementInterfaces,
};

use super::{FromUserProps, Render};

pub trait ComponentTemplate: Sized + 'static {
    type Props<'a>: FromUserProps;

    fn create<Slt>(
        props: Self::Props<'_>,
        slot: Slt,
        access: ElementAccess,
    ) -> impl RootStructureCreate
    where
        Slt: StructureCreate<()>;
}

pub struct Component<T> {
    _el: PhantomData<T>,
    access: ElementAccess,
    slot: ChildBox<()>,
}

impl<T> ElementInterfaces for Component<T>
where
    T: ComponentTemplate,
{
    type Props<'a> = <T as ComponentTemplate>::Props<'a>;
    type SlotData = ();

    fn create<Slt>(
        props: Self::Props<'_>,
        slot: Slt,
        access: ElementAccess,
        ctx: &EMCreateCtx,
    ) -> Self
    where
        Slt: StructureCreate<()>,
    {
        Component {
            slot: ChildBox::new(T::create(props, slot, access.clone()), ctx),
            access,
            _el: PhantomData,
        }
    }

    fn render(&mut self, args: Render) -> crate::Result<()> {
        self.slot.render(args)
    }

    fn spread_event(&mut self, ipe: &IncomingPointerEvent) -> bool {
        self.slot.emit_event(ipe)
    }

    fn on_draw_region_changed(&mut self) {
        let mut dr = Some(self.access.draw_region());
        self.slot
            .layout(|_| dr.take())
            .expect("unexpected layout failure");
    }
}

pub trait RootStructureCreate:
    StructureCreate<(), Target = ElementModel<Self::Element, ()>>
{
    type Element: ElementInterfaces;
}

impl<T, El> RootStructureCreate for T
where
    T: StructureCreate<(), Target = ElementModel<El, ()>>,
    El: ElementInterfaces,
{
    type Element = El;
}
