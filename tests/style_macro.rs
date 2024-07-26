use irisia::style::{ReadStyle, Style, WriteStyle};
use irisia_macros::style;

mod container {
    use irisia::Style;

    #[derive(Debug, Clone, PartialEq)]
    pub struct Padding(pub u32);

    impl From<(u32,)> for Padding {
        fn from((size,): (u32,)) -> Self {
            Padding(size)
        }
    }

    impl Style for Padding {}
}

#[derive(Debug, Clone, PartialEq)]
struct Color(u8, u8, u8);
#[derive(Debug, Clone, PartialEq)]
struct FontSize(u32);
#[derive(Debug, Clone, PartialEq)]
struct Margin(u32);

impl Style for Color {}
impl Style for FontSize {}
impl Style for Margin {}

impl From<(u8, u8, u8)> for Color {
    fn from(rgb: (u8, u8, u8)) -> Self {
        Color(rgb.0, rgb.1, rgb.2)
    }
}

impl From<(u32,)> for FontSize {
    fn from((size,): (u32,)) -> Self {
        FontSize(size)
    }
}

impl From<(u32,)> for Margin {
    fn from((size,): (u32,)) -> Self {
        Margin(size)
    }
}

#[derive(Default, Debug)]
struct TestStyle {
    color: Option<Color>,
    font_size: Option<FontSize>,
    margin: Option<Margin>,
    padding: Option<container::Padding>,
}

impl WriteStyle for TestStyle {
    fn write_style<R>(&mut self, read: &R)
    where
        R: ReadStyle + ?Sized,
    {
        self.color = <_>::from_style(read);
        self.font_size = <_>::from_style(read);
        self.margin = <_>::from_style(read);
        self.padding = <_>::from_style(read);
    }
}

#[test]
fn test_style_macro() {
    let is_large = true;
    let size = "medium";

    let complex_style = style! {
        Color: 0, 255, 0;
        FontSize: 16;
        Margin: 10;

        if is_large {
            FontSize: 24;
        } else {
            FontSize: 16;
        }

        match size {
            "small" => { FontSize: 12; },
            "medium" => { FontSize: 16; },
            "large" => { FontSize: 20; },
            _ => { FontSize: 14; },
        }

        let base_margin = {
            Margin: 10;
        };

        use base_margin;

        in container {
            Padding: 5;
        }

        in * {
            Margin: 15;
        }
    };

    let mut test_style = TestStyle::default();
    test_style.write_style(&complex_style);

    assert_eq!(test_style.color, Some(Color(0, 255, 0)));
    assert_eq!(test_style.font_size, Some(FontSize(16)));
    assert_eq!(test_style.margin, Some(Margin(15)));
    assert_eq!(test_style.padding, Some(container::Padding(5)));
}
