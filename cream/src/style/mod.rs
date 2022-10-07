//! # Style
//! The principle of style is similiar with SkPaint in Skia.
//! ```
//! use champagne::{champagne, RepetitiveStyle};
//!
//! struct MyStyle{
//!     // ...
//! }
//!
//! impl From<(u32, &'static str, u32)> for MyStyle{
//!     // ...
//! }
//!
//! impl RepetitiveStyle for MyStyle{
//!     // ...
//! }
//!
//! champagne!{
//!     body:{
//!         Element{
//!             :style:{
//!                 MyStyle: 1 "string" 20, 2 "string2" 100;
//!             }
//!         }
//!     }
//! }
//! ```

pub mod style_accept;
pub mod style_group;
pub mod style_variant;

pub use style_accept::*;
pub use style_group::*;
pub use style_variant::*;

#[test]
#[allow(unused)]
fn test() {
    struct A;
    struct B;
    #[derive(Clone, Default)]
    struct C;

    struct Foo<'a> {
        style1: Option<&'a A>,
        style2: &'a [A],
        style3: C,
    }

    impl<'a: 'b, 'b> StyleAccept<'a, <Option<&'a A> as StyleVariant<'a>>::InputStyle> for Foo<'b> {
        fn append_borrowed_style(
            &mut self,
            style: &'a <Option<&'a A> as StyleVariant<'a>>::InputStyle,
        ) {
            self.style1 = <_>::from_borrowed_style(style);
        }
        fn append_sliced_style(
            &mut self,
            style: &'a [<Option<&'a A> as StyleVariant<'a>>::InputStyle],
        ) {
            self.style1 = <_>::from_sliced_style(style);
        }
    }

    impl<'a: 'b, 'b> StyleAccept<'a, <ImAStyle<C> as StyleVariant<'a>>::InputStyle> for Foo<'b> {
        fn append_borrowed_style(
            &mut self,
            style: &'a <ImAStyle<C> as StyleVariant<'a>>::InputStyle,
        ) {
            self.style3 = <ImAStyle<_>>::from_borrowed_style(style).0;
        }
        fn append_sliced_style(
            &mut self,
            style: &'a [<ImAStyle<C> as StyleVariant<'a>>::InputStyle],
        ) {
            self.style3 = <ImAStyle<_>>::from_sliced_style(style).0;
        }
    }

    impl<'a> Foo<'a> {
        fn new() -> Self {
            Foo {
                style1: None,
                style2: &[],
                style3: C,
            }
        }
    }

    let mut foo = Foo::new();
    foo.append_borrowed_style(&A);
    foo.append_sliced_style(&[C, C]);
}
