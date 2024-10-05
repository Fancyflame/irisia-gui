use impl_variadics::impl_variadics;

use super::{StructureCreate, VisitBy};
use crate::structure::EMCreateCtx;

impl_variadics! {
    ..=25 "T*" => {
        impl<#(#T0),*> StructureCreate for (#(#T0,)*)
        where
            #(#T0: StructureCreate,)*
        {
            type Target = (#(#T0::Target,)*);

            fn create(self, _ctx: &EMCreateCtx) -> Self::Target {
                (#(self.#index.create(_ctx),)*)
            }
        }

        impl<Cp, #(#T0),*> VisitBy<Cp> for (#(#T0,)*)
        where
            #(#T0: VisitBy<Cp>,)*
        {
            fn visit<V>(&self, _v: &mut V) -> crate::Result<()>
            where
                V: super::Visitor<Cp>
            {
                #(self.#index.visit(_v)?;)*
                Ok(())
            }

            fn visit_mut<V>(&mut self, _v: &mut V) -> crate::Result<()>
            where
                V: super::VisitorMut<Cp>
            {
                #(self.#index.visit_mut(_v)?;)*
                Ok(())
            }
        }
    }
}
