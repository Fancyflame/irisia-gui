use proc_macro2::TokenStream;
use quote::quote;
use stmt::gen_chained;

use super::DomMacro;

mod stmt;

const_quote! {
    const VAR_INPUT_DP = { __irisia_input_dirty_points };
    const PATH_CONTROL_FLOW = { irisia::model::control_flow };
}

impl DomMacro {
    pub fn gen_code(&self) -> TokenStream {
        gen_chained(&self.body)
    }
}
