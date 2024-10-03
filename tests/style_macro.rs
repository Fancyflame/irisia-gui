use irisia::{
    style::{StyleFn, WriteStyle},
    Style,
};
use irisia_macros::style;

mod container {
    use irisia::Style;

    #[derive(Debug, Clone, PartialEq, Style)]
    #[style(all)]
    pub struct Padding(pub u32);
}

#[derive(Debug, Clone, PartialEq, Style)]
#[style(all)]
struct Color(u8, u8, u8);

#[derive(Debug, Clone, PartialEq, Style)]
#[style(all)]
struct FontSize(u32);

#[derive(Debug, Clone, PartialEq, Style)]
#[style(all)]
struct Margin(u32);

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
        R: StyleFn + ?Sized,
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
