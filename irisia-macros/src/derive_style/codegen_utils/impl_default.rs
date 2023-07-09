use quote::quote;
use syn::{Error, Result};

use crate::derive_style::{tag_to_string, variant_analyzer::VariantAnalysis};

use super::CodeGenerator;

impl CodeGenerator {
    pub fn impl_default(&mut self, va: &VariantAnalysis) -> Result<()> {
        let mut body = Vec::new();

        for (tag, field) in va.fields.iter() {
            match &field.default {
                Some(def) => body.push((tag, def)),
                None => {
                    return Err(Error::new_spanned(
                        tag,
                        format!(
                            "cannot implement `Default` for `{}`, because field `{}` does not have default behavior",
                            self.ident, tag_to_string(tag)
                        )
                    ))
                }
            }
        }
        let body = va.field_type.surround(body.into_iter());

        let variant = self.variant.clone();
        self.impl_trait(
            quote!(::std::default::Default),
            quote! {
                fn default() -> Self {
                    #variant #body
                }
            },
        );
        Ok(())
    }
}
