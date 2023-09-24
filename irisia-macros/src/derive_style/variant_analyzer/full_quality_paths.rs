use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, spanned::Spanned, Error, Expr, ExprPath, Member, Result, Token, Type,
};

use crate::derive_style::attributes::Segment;
use crate::derive_style::tag_to_string;

use super::FieldMap;

type SegmentVec = Vec<(Member, FullQualitySegment)>;

#[derive(Debug)]
pub struct FullQualityPaths {
    field_count: usize,
    segs: SegmentVec,
}

#[derive(Debug)]
pub enum FullQualitySegment {
    Required(Type),
    Fn {
        fn_path: ExprPath,
        arg_types: Punctuated<Type, Token![,]>,
    },
    Default(Expr),
}

impl FullQualityPaths {
    pub(super) fn complete(origin_paths: Vec<Vec<Segment>>, fields: &FieldMap) -> Result<Self> {
        let mut output = FullQualityPaths {
            field_count: fields.len(),
            segs: Vec::new(),
        };

        let need_sort = matches!(fields.iter().next(), Some((Member::Unnamed(_), _)));

        for path in origin_paths {
            fill_default(&mut output.segs, &path, fields)?;
            fill_path(&mut output.segs, path, fields)?;

            if need_sort {
                let len = output.segs.len();
                output.segs[len - output.field_count..].sort_by(|(a, _), (b, _)| match (a, b) {
                    (Member::Unnamed(ia), Member::Unnamed(ib)) => ia.index.cmp(&ib.index),
                    _ => unreachable!("expect to be unnamed field"),
                });
            }
        }

        Ok(output)
    }

    pub fn iter(&self) -> impl Iterator<Item = &[(Member, FullQualitySegment)]> {
        let mut iter = if self.field_count == 0 {
            None
        } else {
            assert_eq!(self.segs.len() % self.field_count, 0);
            Some(self.segs.chunks(self.field_count))
        };

        std::iter::from_fn(move || iter.as_mut()?.next())
    }
}

fn fill_default(segs: &mut SegmentVec, path: &[Segment], fields: &FieldMap) -> Result<()> {
    for field in fields {
        if path.iter().any(|seg| seg.tag() == field.0) {
            continue;
        }

        let value = match &field.1.default {
            Some(expr) => FullQualitySegment::Default(expr.clone()),
            None => {
                return Err(Error::new(
                    field.0.span(),
                    format!(
                        "default behavior of field `{}` needs to be specified",
                        tag_to_string(field.0)
                    ),
                ))
            }
        };

        segs.push((field.0.clone(), value));
    }
    Ok(())
}

fn fill_path(segs: &mut SegmentVec, path: Vec<Segment>, fields: &FieldMap) -> Result<()> {
    for seg in path {
        let tag = seg.tag();

        let Some(field) = fields.get(tag)
        else {
            return Err(Error::new(Span::call_site(), format!(
                "field `{}` doesn't exist, but it appears in `from` expression of style derive macros",
                tag_to_string(tag)
            )));
        };

        let value = match seg {
            Segment::Member(m) => (m, FullQualitySegment::Required(field.ty.clone())),
            Segment::Fn {
                bind,
                path,
                arg_types,
            } => (
                bind,
                FullQualitySegment::Fn {
                    fn_path: path,
                    arg_types,
                },
            ),
        };

        segs.push(value);
    }
    Ok(())
}
