use std::{fmt::Write, ops::Deref, rc::Rc};

use crate::primitive::Length;
use anyhow::Error;
use irisia_backend::skia_safe::Color;

#[derive(Clone)]
pub enum StyleValue {
    Color(Color),
    Length(Length),
    Float(f32),
    Bool(bool),
    Ident(Ident),
    Delimiter,
    KeyEq(KeyEq),
}

impl StyleValue {
    pub const fn type_name(&self) -> &'static str {
        match self {
            Self::Color(_) => "color",
            Self::Length(_) => "length",
            Self::Float(_) => "float",
            Self::Bool(_) => "bool",
            Self::Ident(_) => "ident",
            Self::Delimiter => "delimiter",
            Self::KeyEq(_) => "key equal",
        }
    }
}

#[derive(Clone, Copy)]
pub struct Delimiter;

#[derive(Clone, Copy)]
pub struct Eof(());

pub trait ParseStyleValue: Clone {
    fn try_parse(this: &[StyleValue]) -> Option<(Self, &[StyleValue])>;
    fn type_name() -> &'static str;
}

macro_rules! impl_psv {
    ($($Var:ident $type_name:literal $Type:ty,)*) => {
        $(
            impl ParseStyleValue for $Type {
                fn try_parse(this: &[StyleValue]) -> Option<(Self, &[StyleValue])> {
                    if let [StyleValue::$Var(val), ref rest @ ..] = this {
                        Some((val.clone(), rest))
                    } else {
                        None
                    }
                }

                fn type_name() -> &'static str {
                    $type_name
                }
            }
        )*
    };
}

impl_psv! {
    Color      "color"      Color ,
    Length     "length"     Length,
    Float      "float"      f32   ,
    Bool       "bool"       bool  ,
    Ident      "ident"      Ident ,
}

impl ParseStyleValue for Delimiter {
    fn try_parse(this: &[StyleValue]) -> Option<(Self, &[StyleValue])> {
        if let [StyleValue::Delimiter, ref rest @ ..] = this {
            Some((Delimiter, rest))
        } else {
            None
        }
    }

    fn type_name() -> &'static str {
        "delimiter"
    }
}

impl ParseStyleValue for Eof {
    fn try_parse(this: &[StyleValue]) -> Option<(Self, &[StyleValue])> {
        if this.is_empty() {
            Some((Eof(()), this))
        } else {
            None
        }
    }

    fn type_name() -> &'static str {
        "end of stream"
    }
}

impl ParseStyleValue for StyleValue {
    fn try_parse(this: &[StyleValue]) -> Option<(Self, &[StyleValue])> {
        match this.split_first() {
            Some((StyleValue::KeyEq(_), _)) | None => None,
            Some((value, rest)) => Some((value.clone(), rest)),
        }
    }

    fn type_name() -> &'static str {
        "any value"
    }
}

impl<T> ParseStyleValue for (KeyEq, T)
where
    T: ParseStyleValue,
{
    fn try_parse(this: &[StyleValue]) -> Option<(Self, &[StyleValue])> {
        let [StyleValue::KeyEq(key), rest @ ..] = this else {
            return None;
        };

        let (value, rest) = T::try_parse(rest)?;
        Some(((key.clone(), value), rest))
    }

    fn type_name() -> &'static str {
        "key-value"
    }
}

macro_rules! impl_from {
    ($($Var:ident $Type:ty,)*) => {
        $(
            impl From<$Type> for StyleValue {
                fn from(value: $Type) -> Self {
                    Self::$Var(value)
                }
            }
        )*
    };
}

impl_from! {
    Color Color,
    Length Length,
    Float f32,
    Bool bool,
    KeyEq KeyEq,
}

impl From<&'static str> for StyleValue {
    fn from(value: &'static str) -> Self {
        Self::Ident(Ident::new(value))
    }
}

impl From<Delimiter> for StyleValue {
    fn from(_: Delimiter) -> Self {
        Self::Delimiter
    }
}

#[derive(Clone)]
pub struct Ident(IdentInner);

#[derive(Clone)]
enum IdentInner {
    Borrowed(&'static str),
    Rc(Rc<str>),
}

impl Ident {
    pub const fn new(ident: &'static str) -> Self {
        Self(IdentInner::Borrowed(ident))
    }

    /// Do not use this unless for debug usage
    pub fn new_debug(ident: &str) -> Self {
        Self(IdentInner::Rc(ident.into()))
    }

    pub fn error(&self, expected: &[&str]) -> Error {
        let mut s = "expected identifier".to_string();
        if let Some(last_index) = expected.len().checked_sub(1) {
            for (index, &ident) in expected.iter().enumerate() {
                let spliter = if index == 0 {
                    " "
                } else if index == last_index {
                    " or "
                } else {
                    ", "
                };
                write!(&mut s, "{spliter}`{ident}`").unwrap();
            }
        } else {
            s += "matches nothing";
        };

        write!(&mut s, ", found {}`", &**self).unwrap();
        Error::msg(s)
    }
}

impl Deref for Ident {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        match &self.0 {
            &IdentInner::Borrowed(b) => b,
            IdentInner::Rc(r) => r,
        }
    }
}

#[derive(Clone)]
pub struct KeyEq(Ident);

impl KeyEq {
    pub const fn new(key: Ident) -> Self {
        Self(key)
    }
}

impl Deref for KeyEq {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
