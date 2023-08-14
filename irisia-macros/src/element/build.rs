use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::ParseStream, Result};

use crate::expr::state_block::{parse_stmts, stmts_to_tokens};

use super::ElementCodegen;

pub fn build(input: ParseStream) -> Result<TokenStream> {
    let stmts = parse_stmts::<ElementCodegen>(input)?;
    let stmts_tokened = stmts_to_tokens(&stmts);
    Ok(quote! {{
        use irisia::structure::StructureBuilder;
        #stmts_tokened
    }})
}
