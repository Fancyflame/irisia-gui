pub struct ForLoopIterItemAsKey<I, F> {
    iter: I,
    map: F,
}

impl<I, F, K, T> Iterator for ForLoopIterItemAsKey<I, F>
where
    I: Iterator<Item = K>,
    K: Clone,
    F: FnMut(K) -> T,
{
    type Item = (K, T);
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|item| (item.clone(), (self.map)(item)))
    }
}

pub fn for_loop_iter_item_as_key<I, F>(iter: I, map: F) -> ForLoopIterItemAsKey<I, F> {
    ForLoopIterItemAsKey { iter, map }
}
