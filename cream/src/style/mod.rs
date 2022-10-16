pub mod style_table;

pub use style_table::StyleTable;

pub trait Style {}

pub trait ApplyStyle {
    fn apply_style(&self, table: &mut StyleTable);
}
