use std::marker::PhantomData;

use crate::{
    application::event_comp::IncomingPointerEvent,
    coerce_signal,
    el_model::{EMCreateCtx, ElementAccess},
    hook::{Provider, ProviderObject, Signal},
    model::{
        iter::{basic::ModelBasicMapper, VisitModel},
        DesiredVModel, VModel,
    },
    ElementInterfaces,
};

use super::{children_utils::ChildrenUtils, deps::AsEmptyProps, Render};

pub trait ComponentTemplate: Sized + 'static {
    type Props: AsEmptyProps;

    fn create<Slt>(
        props: &Self::Props,
        access: ElementAccess,
        slot: ProviderObject<Slt>,
    ) -> ProviderObject<impl DesiredVModel<ModelBasicMapper> + 'static>
    where
        Slt: DesiredVModel<ModelBasicMapper> + 'static;
}

pub struct Component<T> {
    _el: PhantomData<T>,
    slot: Signal<dyn VisitModel<ModelBasicMapper>>,
}

impl<T> ElementInterfaces for Component<T>
where
    T: ComponentTemplate,
{
    type Props = <T as ComponentTemplate>::Props;
    type AcceptChild = ModelBasicMapper;

    fn create<Slt>(
        props: &Self::Props,
        access: ElementAccess,
        slot: crate::hook::ProviderObject<Slt>,
        ctx: &EMCreateCtx,
    ) -> Self
    where
        Slt: DesiredVModel<Self::AcceptChild> + 'static,
    {
        let vmodel = T::create(props, access.clone(), slot);
        let ctx_cloned = ctx.clone();
        let slot = Signal::builder(vmodel.read().create(ctx));
        let slot = slot
            .dep(
                move |mut slot, vmodel| {
                    vmodel.update(&mut *slot, &ctx_cloned);
                },
                vmodel,
            )
            .build();

        Component {
            slot: coerce_signal!(slot),
            _el: PhantomData,
        }
    }

    fn render(&mut self, args: Render) -> crate::Result<()> {
        self.slot.write().render(args)
    }

    fn spread_event(&mut self, ipe: &IncomingPointerEvent) -> bool {
        self.slot.write().emit_event(ipe)
    }

    fn on_draw_region_change(&mut self) {
        /*let mut dr = Some(self.access.draw_region());
        self.slot
            .layout(|_| dr.take())
            .expect("unexpected layout failure");*/
        unimplemented!()
    }
}
