use super::StyleSource;

pub struct Splited<A, B> {
    split_at: f32,
    first: A,
    second: B,
}

impl<A, B> StyleSource for Splited<A, B>
where
    A: StyleSource,
    B: StyleSource,
{
    fn get_style<'a>(&'a self, name: &str, prog: f32) -> Option<&'a [super::StyleValue]> {
        if prog < self.split_at {
            self.first.get_style(name, prog / self.split_at)
        } else {
            self.second
                .get_style(name, (prog - self.split_at) / (1 - self.split_at))
        }
    }
}
