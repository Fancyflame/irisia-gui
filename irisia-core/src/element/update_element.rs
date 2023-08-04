use std::sync::Arc;

use crate::{
    application::content::GlobalContent,
    dom::{
        children::{ChildrenBox, RenderObject, SetChildren},
        update::ApplyGlobalContent,
    },
    structure::MapVisit,
    update_with::SpecificUpdate,
    UpdateWith,
};

use super::ComputeSize;

pub struct UpdateOptions<'a, Pr, Sty, Ch> {
    pub props: Pr,
    pub styles: &'a Sty,
    pub children: Ch,
    pub updater: UpdateElement<'a>,
}

pub struct UpdateElement<'a> {
    set_children: SetChildren<'a>,
    global_content: ApplyGlobalContent<'a>,
    compute_size: &'a mut ComputeSize,
}

impl<'a> UpdateElement<'a> {
    pub(crate) fn new(
        set_children: SetChildren<'a>,
        gc: &'a Arc<GlobalContent>,
        compute_size: &'a mut ComputeSize,
    ) -> Self {
        Self {
            set_children,
            global_content: ApplyGlobalContent(gc),
            compute_size,
        }
    }

    pub fn set_children<'b, T>(
        &'b mut self,
        children: T,
        equality_matters: &mut bool,
    ) -> &'b mut <T::Output as SpecificUpdate>::UpdateTo
    where
        T: MapVisit<ApplyGlobalContent<'a>>,
        T::Output: SpecificUpdate,
        <T::Output as SpecificUpdate>::UpdateTo: RenderObject + UpdateWith<T::Output> + 'static,
    {
        let cb = match &mut self.set_children {
            SetChildren::Create(opt) => {
                opt.insert(ChildrenBox::create_with(children.map(&self.global_content)))
            }
            SetChildren::Update(upd) => {
                *equality_matters &=
                    upd.update_with(children.map(&self.global_content), *equality_matters);
                upd
            }
        };

        cb.as_render_object()
            .as_any()
            .downcast_mut()
            .unwrap_or_else(|| unreachable!())
    }

    pub fn set_compute_size(&mut self, compute_size: ComputeSize) {
        *self.compute_size = compute_size;
    }
}
