use crate::{
    application::event_comp::IncomingPointerEvent,
    element::Render,
    model::iter::{ModelMapper, VisitModel},
    primitive::Region,
    Result,
};

pub trait ChildrenUtils<M>: VisitModel<M>
where
    M: ModelMapper,
{
    fn render(&mut self, args: Render) -> Result<()> {
        let mut result = Ok(());
        self.visit_mut(&mut |mut basic| {
            let prev = std::mem::replace(&mut result, Ok(()));
            result = prev.and(basic.as_mut().dyn_render(args));
        });
        result
    }

    fn emit_event(&mut self, ipe: &IncomingPointerEvent) -> bool {
        let mut childern_entered = false;
        self.visit_mut(&mut |mut basic| {
            childern_entered |= basic.as_mut().dyn_on_pointer_event(ipe);
        });
        childern_entered
    }

    fn layout<F>(&mut self, mut layout: F)
    where
        F: FnMut(&M::MapMut<'_>) -> Option<Region>,
    {
        self.visit_mut(&mut |mut refm| {
            let region = layout(&refm);
            refm.as_mut().dyn_layout(region);
        });
    }

    fn compute_len(&self) -> usize {
        let mut len = 0;
        self.visit(&mut |_| len += 1);
        len
    }
}

impl<M, T> ChildrenUtils<M> for T
where
    T: VisitModel<M> + ?Sized,
    M: ModelMapper,
{
}
