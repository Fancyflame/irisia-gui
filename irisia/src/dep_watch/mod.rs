pub use self::{
    bitset::{Bitset, DependencyIndexes},
    data_source::DataSource,
    dep_stack::DependentStack,
};

pub mod bitset;
pub mod data_source;
pub mod dep_stack;
pub mod inferer;

// Utils

pub const fn bitset_width(field_count: u32) -> usize {
    field_count.div_ceil(usize::BITS) as _
}

#[cfg(test)]
mod test {
    use super::{DataSource, DependentStack};

    #[test]
    fn test_bitset() {
        assert!(usize::BITS >= 8);

        let stack: DependentStack<[usize; 2]> = DependentStack::new();

        let biggest = usize::BITS * 2 - 1;
        let middle = usize::BITS * 3 / 2;
        let smallest = usize::BITS / 3;

        let mut src = DataSource::new(0, &stack);

        stack.scoped(biggest, || {
            src.read();
        });

        stack.scoped(smallest, || {
            src.read();
        });

        stack.scoped(middle, || {
            src.read();
        });

        let _ = src.update();
        let mut di = stack.get_update_list(true);

        assert!(di.step_if(smallest));
        assert!(di.step_if(middle));
        assert!(di.step_if(biggest));
        assert!(di.peek().is_none());
    }
}
