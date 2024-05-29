use std::time::Duration;

use crate::{
    application::event_comp::IncomingPointerEvent,
    data_flow::ReadWire,
    el_model::{layer::LayerRebuilder, EMCreateCtx, ElInputWatcher, ElementAccess, SharedEM},
    primitive::Region,
    structure::{ChildBox, StructureCreate},
    ElementInterfaces,
};

pub trait ComponentTemplate: Sized + 'static {
    type Props<'a>;

    fn create<Slt>(
        props: Self::Props<'_>,
        slot: Slt,
        access: ElementAccess,
        watch_input: CompInputWatcher<Self>,
    ) -> (
        Self,
        impl StructureCreate<Target = SharedEM<impl ElementInterfaces>>,
    )
    where
        Slt: StructureCreate;
}

pub struct Component<T> {
    inner: T,
    slot: ChildBox,
}

impl<T> ElementInterfaces for Component<T>
where
    T: ComponentTemplate,
{
    type Props<'a> = <T as ComponentTemplate>::Props<'a>;

    fn create<Slt>(
        props: Self::Props<'_>,
        slot: Slt,
        access: ElementAccess,
        watch_input: ElInputWatcher<Self>,
        ctx: &EMCreateCtx,
    ) -> Self
    where
        Slt: StructureCreate,
    {
        let (inner, slot) =
            <T as ComponentTemplate>::create(props, slot, access, CompInputWatcher(watch_input));

        Component {
            inner,
            slot: ChildBox::new(slot, ctx),
        }
    }

    fn render(&mut self, lr: &mut LayerRebuilder, interval: Duration) -> crate::Result<()> {
        self.slot.render(lr, interval)
    }

    fn children_emit_event(&mut self, ipe: &IncomingPointerEvent) -> bool {
        self.slot.emit_event(ipe)
    }

    fn set_draw_region(&mut self, dr: Region) {
        let mut dr = Some(dr);
        self.slot
            .layout(|_| dr.take())
            .expect("unexpected layout failure");
    }
}

pub struct CompInputWatcher<El>(ElInputWatcher<Component<El>>);

impl<El: ComponentTemplate> CompInputWatcher<El> {
    pub fn watch<U, F>(&self, watch: ReadWire<U>, mut func: F)
    where
        U: 'static,
        F: FnMut(&mut El, &U) + 'static,
    {
        self.0.watch(watch, move |comp, watch_data| {
            func(&mut comp.inner, watch_data)
        });
    }

    pub fn invoke<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&El) -> R,
    {
        self.0.invoke(|comp| f(&comp.inner))
    }

    pub fn invoke_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut El) -> R,
    {
        self.0.invoke_mut(|comp| f(&mut comp.inner))
    }
}
