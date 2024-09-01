use anyhow::{anyhow, Result};

use crate::style::value::Eof;

use super::{value::ParseStyleValue, StyleValue};

#[derive(Clone)]
#[must_use]
pub struct ParseRule<'a> {
    rule_name: &'a str,
    stream: &'a [StyleValue],
}

impl<'a> ParseRule<'a> {
    pub(super) fn new(rule_name: &'a str, stream: &'a [StyleValue]) -> Self {
        Self { rule_name, stream }
    }

    pub fn name(&self) -> &'a str {
        self.rule_name
    }

    pub fn try_parse<T: ParseStyleValue>(&mut self) -> Option<T> {
        let (result, rest) = T::try_parse(self.stream)?;
        self.stream = rest;
        Some(result)
    }

    pub fn parse<T: ParseStyleValue>(&mut self) -> Result<T> {
        self.try_parse().ok_or_else(|| {
            let found = if let Some(first) = self.stream.first() {
                first.type_name()
            } else {
                Eof::type_name()
            };
            anyhow!("expect {}, found {found}", T::type_name())
        })
    }

    pub fn peek<T: ParseStyleValue>(&self) -> Option<T> {
        T::try_parse(self.stream).map(|(x, _)| x)
    }

    pub fn is_empty(&self) -> bool {
        self.stream.is_empty()
    }

    pub fn ignore_rest(mut self) {
        self.stream = &[];
    }
}

impl Drop for ParseRule<'_> {
    fn drop(&mut self) {
        if !self.is_empty() {
            panic!(
                "drop check failed: there still values remain in the stream. \
                if it is expected, call `Self::ignore_rest` explictly."
            );
        }
    }
}
