use reader::ParseRule;

pub use value::StyleValue;

mod reader;
pub mod value;

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

impl StyleFn for () {
    fn read(&self, _: ReadStyleFn) {}
}
