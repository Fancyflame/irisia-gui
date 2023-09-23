use proc_macro2::TokenStream;
use syn::{parse::ParseStream, Result};

use crate::expr::state_block::{parse_stmts, stmts_to_tokens};

use super::ElementCodegen;

pub fn build(input: ParseStream) -> Result<TokenStream> {
    let stmts = parse_stmts::<ElementCodegen>(input)?;
    Ok(stmts_to_tokens(&stmts))
}
