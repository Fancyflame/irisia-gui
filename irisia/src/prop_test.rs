use crate as irisia;
use irisia_macros::Style;

#[irisia::props(PersonInfoUpdater)]
/*  除此之外可添加可访问性。例如上面宏将扩展为下面，访问性默认与
    宏所在结构体一致。
    另外，props宏只允许应用于struct，enum和union都不行。
    #[irisia::props(PersonInfoUpdater, vis = "pub")]
*/
pub struct PersonInfo {
    #[props(updater, must_init)]
    name: String,

    #[props(updater, default)]
    job_codes: Vec<u32>,

    #[props(must_init)]
    rank: u8,

    #[props(default)]
    age: u16,

    #[props(with(Gender::from_str, &str))]
    gender: Gender,

    #[props(style_reader, default_with = "MyStyle::new")]
    style: MyStyle,
}

#[derive(Style, Clone)]
struct MyStyle;

impl MyStyle {
    fn new() -> Self {
        MyStyle
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Gender {
    Male,
    Female,
    Unknown,
}

impl Gender {
    fn from_str(string: &str) -> Gender {
        todo!()
    }
}

// ****************
// 展开为
// ****************

macro_rules! expanded {
    () => {
        这个阻止格式化
        pub struct PersonInfoUpdater<T1 = (), T2 = (), T3 = (), T4 = (), T5 = (), T6 = ()> {
            name: T1,
            job_codes: T2,
            rank: T3,
            age: T4,
            gender: T5,
            style: T6,
        }

        impl ::std::default::Default for PersonInfoUpdater<(), (), (), (), (), ()> {
            fn default() {
                PersonInfoUpdater {
                    name:      (),
                    job_codes: (),
                    rank:      (),
                    age:       (),
                    gender:    (),
                    style:     (),
                }
            }
        }

        impl<T1, T2, T3, T4, T5, T6> PersonInfoUpdater<T1, T2, T3, T4, T5, T6> {
            pub fn name<U>      (self, value: U) -> PersonInfoUpdater<U, T2, T3, T4, T5, T6> {
                PersonInfoUpdater {
                    name:      (value,),
                    job_codes: self.job_codes,
                    rank:      self.rank,
                    age:       self.age,
                    gender:    self.gender,
                    style:     self.style,
                }
            }

            pub fn job_codes<U> (self, value: U) -> PersonInfoUpdater<T1, U, T3, T4, T5, T6> {
                PersonInfoUpdater {
                    name:      self.name,
                    job_codes: (value,),
                    rank:      self.rank,
                    age:       self.age,
                    gender:    self.gender,
                    style:     self.style,
                }
            }

            pub fn rank<U>      (self, value: U) -> PersonInfoUpdater<T1, T2, U, T4, T5, T6> {
                PersonInfoUpdater {
                    name:      self.name,
                    job_codes: self.job_codes,
                    rank:      (value,),
                    age:       self.age,
                    gender:    self.gender,
                    style:     self.style,
                }
            }

            pub fn age<U>       (self, value: U) -> PersonInfoUpdater<T1, T2, T3, U, T5, T6> {
                PersonInfoUpdater {
                    name:      self.name,
                    job_codes: self.job_codes,
                    rank:      self.rank,
                    age:       (value,),
                    gender:    self.gender,
                    style:     self.style,
                }
            }

            pub fn gender<U>    (self, value: U) -> PersonInfoUpdater<T1, T2, T3, T4, U, T6> {
                PersonInfoUpdater {
                    name:      self.name,
                    job_codes: self.job_codes,
                    rank:      self.rank,
                    age:       self.age,
                    gender:    (value,),
                    style:     self.style,
                }
            }

            pub fn style<U>     (self, style: U) -> PersonInfoUpdater<T1, T2, T3, T4, T5, U> {
                PersonInfoUpdater {
                    name:      self.name,
                    job_codes: self.job_codes,
                    rank:      self.rank,
                    age:       self.age,
                    gender:    self.gender,
                    style:     (value,),
                }
            }
        }

        impl<T1, T2, T3, T4, T5, T6> irisia::UpdateWith<PersonInfoUpdater<T1, T2, T3, T4, T5, T6>>
            for PersonInfo
        where
            irisia::element::props::CallUpdater:        irisia::element::props::HelpUpdate<String,   T1, Def = irisia::element::props::ValueInitialized<String>>,
            irisia::element::props::CallUpdater:        irisia::element::props::HelpUpdate<Vec<u32>, T2>,
            irisia::element::props::MoveOwnership:      irisia::element::props::HelpUpdate<u8,       T3, Def = irisia::element::props::ValueInitialized<u8>>,
            irisia::element::props::MoveOwnership:      irisia::element::props::HelpUpdate<u16,      T4>,
            fn(&str) -> Gender:                         irisia::element::props::HelpUpdate<Gender,   T5, Def = irisia::element::props::ValueInitialized<Gender>>,
            irisia::element::props::ReadStyle:          irisia::element::props::HelpUpdate<MyStyle,  T6>,
        {
            fn update_with(
                &mut self,
                updater: PersonInfoUpdater<T1, T2, T3, T4, T5, T6>,
                mut equality_matters: bool,
            ) -> bool {
                equality_matters &= irisia::element::props::CallUpdater     .update(&mut self.name,      updater.name,      equality_matters);
                equality_matters &= irisia::element::props::CallUpdater     .update(&mut self.job_codes, updater.job_codes, equality_matters);
                equality_matters &= irisia::element::props::MoveOwnership   .update(&mut self.rank,      updater.rank,      equality_matters);
                equality_matters &= irisia::element::props::MoveOwnership   .update(&mut self.age,       updater.age,       equality_matters);
                equality_matters &= (Gender::from_str as fn(&str) -> Gender).update(&mut self.gender,    updater.gender,    equality_matters);
                equality_matters &= irisia::element::props::ReadStyle       .update(&mut self.style,     updater.style,     equality_matters);
                equality_matters
            }

            fn create_with(updater: PersonInfoUpdater<T1, T2, T3, T4, T5, T6>) -> Self {
                Self {
                    name:      irisia::element::props::CallUpdater     .create(updater.name)     .must_be_initialized(),
                    job_codes: irisia::element::props::CallUpdater     .create(updater.job_codes).with_defaulter(Default::default),
                    rank:      irisia::element::props::MoveOwnership   .create(updater.rank)     .must_be_initialized(),
                    age:       irisia::element::props::MoveOwnership   .create(updater.age)      .with_defaulter(Default::default),
                    gender:    (Gender::from_str as fn(&str) -> Gender).create(updater.gender)   .must_be_initialized(),
                    style:     irisia::element::props::ReadStyle       .create(updater.style)    .with_defaulter(MyStyle::new),
                }
            }
        }
    }
}
