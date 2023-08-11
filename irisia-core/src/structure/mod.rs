use crate::update_with::UpdateWith;

pub mod branch;
pub mod chain;
pub mod empty;
pub mod once;
pub mod repeating;
pub(crate) mod slot;

pub trait VisitLen: Sized {
    fn len(&self) -> usize;
}

macro_rules! impl_structure {
    ($Visit:ident $Visitor:ident $visit:ident $visit_with_control_flow:ident $($mut:ident)?) => {
        pub trait $Visit<V>: VisitLen {
            fn $visit_with_control_flow(& $($mut)? self, visitor: &mut V, control: &mut ControlFlow);

            fn $visit(& $($mut)? self, visitor: &mut V) {
                self.$visit_with_control_flow(visitor, &mut ControlFlow::Continue)
            }
        }

        pub trait $Visitor<T>: Sized {
            fn $visit(&mut self, data: & $($mut)? T, control: &mut ControlFlow);
        }
    };
}

pub trait MapVisit<V> {
    type Output;
    fn map(self, visitor: &V) -> Self::Output;
}

pub trait MapVisitor<T> {
    type Output;
    fn map_visit(&self, data: T) -> Self::Output;
}

impl_structure!(Visit Visitor visit visit_with_control_flow);
impl_structure!(VisitMut VisitorMut visit_mut visit_mut_with_control_flow mut);

#[derive(Debug, Clone, Copy)]
pub enum ControlFlow {
    Continue,
    Exit,
}

impl ControlFlow {
    pub fn set_exit(&mut self) {
        *self = Self::Exit;
    }

    fn should_exit(&self) -> bool {
        matches!(self, Self::Exit)
    }
}
