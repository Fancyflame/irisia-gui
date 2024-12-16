use std::marker::PhantomData;

use crate::{
    application::event_comp::IncomingPointerEvent,
    el_model::{EMCreateCtx, ElementAccess},
    hook::ProviderObject,
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
    ) -> impl DesiredVModel<ModelBasicMapper>
    where
        Slt: DesiredVModel<ModelBasicMapper>;
}

pub struct Component<T> {
    _el: PhantomData<T>,
    access: ElementAccess,
    slot: Box<dyn VisitModel<ModelBasicMapper>>,
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
        Slt: crate::model::DesiredVModel<Self::AcceptChild>,
    {
        let vmodel = T::create(props, access.clone(), slot);
        Component {
            slot: Box::new(vmodel.create(ctx)),
            access,
            _el: PhantomData,
        }
    }

    fn render(&mut self, args: Render) -> crate::Result<()> {
        self.slot.as_mut().render(args)
    }

    fn spread_event(&mut self, ipe: &IncomingPointerEvent) -> bool {
        self.slot.as_mut().emit_event(ipe)
    }

    fn on_draw_region_change(&mut self) {
        /*let mut dr = Some(self.access.draw_region());
        self.slot
            .layout(|_| dr.take())
            .expect("unexpected layout failure");*/
        unimplemented!()
    }
}
