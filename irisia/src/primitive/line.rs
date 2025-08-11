pub struct Line<T> {
    pub start: T,
    pub end: T,
}

impl_mul_dimensions!(Line start end);
