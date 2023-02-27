cream_macros::uninit_type! {
    pub type Test;
    pub type Test2;
}

fn main() {
    mod a {
        cream_macros::set_type! {
            type super::Test = String;
        }
    }
}
