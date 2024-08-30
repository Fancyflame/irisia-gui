use std::borrow::Cow;

use crate::primitive::Length;
use irisia_backend::skia_safe::Color;
use reader::ParseRule;

mod reader;

pub type ReadStyleFn<'a> = &'a mut dyn FnMut(ParseRule);

pub struct StyleBuffer<'a>(ReadStyleFn<'a>);

impl StyleBuffer<'_> {
    pub fn write(&mut self, rule_name: &str, body: &[StyleValue]) {
        (self.0)(ParseRule::new(rule_name, body))
    }
}

pub trait StyleFn {
    fn read(&self, f: ReadStyleFn);
}

impl<F> StyleFn for F
where
    F: Fn(StyleBuffer),
{
    fn read(&self, f: ReadStyleFn) {
        self(StyleBuffer(f))
    }
}

type Float = f32;
pub type Identifier = Cow<'static, str>;

#[derive(Clone)]
pub enum StyleValue {
    Color(Color),
    Length(Length),
    Float(Float),
    Identifier(Identifier),
    Delimiter,
}

#[derive(Clone, Copy)]
pub struct Delimiter;

#[derive(Clone, Copy)]
pub struct Eof;

impl StyleValue {
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Color(_) => "color",
            Self::Length(_) => "length",
            Self::Float(_) => "float",
            Self::Identifier(_) => "identifier",
            Self::Delimiter => "delimiter",
        }
    }
}

pub trait ParseStyleValue: Clone {
    fn try_parse(this: Option<&StyleValue>) -> Option<&Self>;
    fn type_name() -> &'static str;
}

macro_rules! impl_psv {
    ($($Var:ident $type_name:ident,)*) => {
        $(
            impl ParseStyleValue for $Var {
                fn try_parse(this: Option<&StyleValue>) -> Option<&Self> {
                    if let StyleValue::$Var(val) = this? {
                        Some(val)
                    } else {
                        None
                    }
                }

                fn type_name() -> &'static str {
                    stringify!($type_name)
                }
            }
        )*
    };
}

impl_psv! {
    Color color,
    Length length,
    Float float,
    Identifier identifier,
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
