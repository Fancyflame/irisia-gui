use std::{borrow::Cow, ops::Deref};

use crate::primitive::Length;
use irisia_backend::skia_safe::Color;

#[derive(Clone)]
pub enum StyleValue {
    Color(Color),
    Length(Length),
    Float(f32),
    Bool(bool),
    Identifier(Identifier),
    Delimiter,
}

impl StyleValue {
    pub const fn type_name(&self) -> &'static str {
        match self {
            Self::Color(_) => "color",
            Self::Length(_) => "length",
            Self::Float(_) => "float",
            Self::Bool(_) => "bool",
            Self::Identifier(_) => "identifier",
            Self::Delimiter => "delimiter",
        }
    }
}

#[derive(Clone, Copy)]
pub struct Delimiter;

#[derive(Clone, Copy)]
pub struct Eof;

#[derive(Clone)]
pub struct Identifier(Cow<'static, str>);

impl Identifier {
    pub const fn new(ident: &'static str) -> Self {
        Self(Cow::Borrowed(ident))
    }

    /// Do not use this unless for debug usage
    pub fn new_debug(ident: String) -> Self {
        Self(Cow::Owned(ident))
    }

    pub fn is_strict_ident(&self) -> bool {
        let mut trailing = false;
        self.0.chars().all(|ch| {
            let match_alpha = matches!(ch, '_' | 'a'..='z');
            let match_numeric = trailing && matches!(ch, '0'..='9');
            trailing = true;
            match_alpha || match_numeric
        })
    }
}

impl Deref for Identifier {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait ParseStyleValue: Clone {
    fn try_parse(this: Option<&StyleValue>) -> Option<&Self>;
    fn type_name() -> &'static str;
}

macro_rules! impl_psv {
    ($($Var:ident $type_name:literal $Type:ty,)*) => {
        $(
            impl ParseStyleValue for $Type {
                fn try_parse(this: Option<&StyleValue>) -> Option<&Self> {
                    if let StyleValue::$Var(val) = this? {
                        Some(val)
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
    Color      "color"      Color     ,
    Length     "length"     Length    ,
    Float      "float"      f32       ,
    Bool       "bool"       bool      ,
    Identifier "identifier" Identifier,
}

impl ParseStyleValue for Delimiter {
    fn try_parse(this: Option<&StyleValue>) -> Option<&Self> {
        if let StyleValue::Delimiter = this? {
            Some(&Delimiter)
        } else {
            None
        }
    }

    fn type_name() -> &'static str {
        "delimiter"
    }
}

impl ParseStyleValue for Eof {
    fn try_parse(this: Option<&StyleValue>) -> Option<&Self> {
        if this.is_none() {
            Some(&Self)
        } else {
            None
        }
    }

    fn type_name() -> &'static str {
        "end of stream"
    }
}

impl ParseStyleValue for StyleValue {
    fn try_parse(this: Option<&StyleValue>) -> Option<&Self> {
        this
    }

    fn type_name() -> &'static str {
        "any value"
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
}

impl From<&'static str> for StyleValue {
    fn from(value: &'static str) -> Self {
        Self::Identifier(Identifier::new(value))
    }
}

impl From<Delimiter> for StyleValue {
    fn from(_: Delimiter) -> Self {
        Self::Delimiter
    }
}
