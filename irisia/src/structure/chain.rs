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

        impl<#(#T0),*> VisitBy for (#(#T0,)*)
        where
            #(#T0: VisitBy,)*
        {
            fn visit<V>(&self, _v: &mut V) -> crate::Result<()>
            where
                V: super::Visitor
            {
                #(self.#index.visit(_v)?;)*
                Ok(())
            }

            fn len(&self) -> usize {
                0 #(+ self.#index.len())*
            }
        }
    }
}
