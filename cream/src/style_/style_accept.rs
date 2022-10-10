pub trait StyleAccept<'a, S> {
    fn append_borrowed_style(&mut self, style: &'a S);
    fn append_sliced_style(&mut self, style: &'a [S]);
}
