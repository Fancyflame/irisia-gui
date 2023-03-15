use cream_macros::Style;

fn main() {}

#[derive(Style, Clone)]
enum MyStyle {
    #[cream(from)]
    XY { x: u32, y: u32 },

    #[cream(from)]
    Quant {
        left: u32,
        top: u32,
        right: u32,
        bottom: u32,
    },
}

#[derive(Style, Clone)]
#[cream(from = "t, l[, b, r] | l")]
struct StructualStyle {
    l: u32,

    #[cream(default)]
    r: u32,

    #[cream(default)]
    t: u32,

    #[cream(default)]
    b: u32,
}
