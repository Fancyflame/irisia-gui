use crate as irisia;
use irisia_macros::Style;

#[derive(UpdateFrom)]
pub struct PersonInfo {
    #[prop(updater, must_init)]
    name: String,

    #[prop(updater, default)]
    job_codes: Vec<u32>,

    #[prop(must_init)]
    rank: u8,

    #[prop(default)]
    age: u16,

    // if no lifetime parameter, alias to
    // #[prop(with(Gender::from_str, &'static str))]
    #[prop(with(Gender::from_str, for<'a> &'a str))]
    gender: Gender,

    #[prop(style_reader, default_with = "MyStyle::new")]
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
// expand to
// ****************

mod __random_mod_name_1ea9730f9db2 {
    use super::*;
    use crate as irisia;
    use irisia::element::props::*;
    use irisia::style::StyleReader;

    pub struct AnonymousStruct<T1, T2, T3, T4, T5, T6> {
        name: T1,
        job_codes: T2,
        rank: T3,
        age: T4,
        gender: T5,
        style: T6,
    }

    impl<T1, T2, T3, T4, T5, T6> UpdateFrom<AnonymousStruct<T1, T2, T3, T4, T5, T6>>
        for super::PersonInfo
    where
        CallUpdater:        HelpUpdate<String,   T1, Def = MustBeInitialized<String>>,
        CallUpdater:        HelpUpdate<Vec<u32>, T2>,
        MoveOwnership:      HelpUpdate<u8,       T3, Def = MustBeInitialized<u8>>,
        MoveOwnership:      HelpUpdate<u16,      T4>,
        fn(&str) -> Gender: HelpUpdate<Gender,   T5, Def = MustBeInitialized<Gender>>,
        ReadStyle:          HelpUpdate<MyStyle,  T6>,
    {
        fn state_update(
            &mut self,
            updater: AnonymousStruct<T1, T2, T3, T4, T5, T6>,
            mut equality_matters: bool,
        ) -> bool {
            equality_matters &= CallUpdater  .update(&mut self.name,      updater.name,      equality_matters);
            equality_matters &= CallUpdater  .update(&mut self.job_codes, updater.job_codes, equality_matters);
            equality_matters &= MoveOwnership.update(&mut self.rank,      updater.rank,      equality_matters);
            equality_matters &= MoveOwnership.update(&mut self.age,       updater.age,       equality_matters);
            equality_matters &= (Gender::from_str as fn(&str) -> Gender)
                                             .update(&mut self.gender,    updater.gender,    equality_matters);
            equality_matters &= ReadStyle    .update(&mut self.style,     updater.style,     equality_matters);
            equality_matters
        }

        fn state_create(updater: AnonymousStruct<T1, T2, T3, T4, T5, T6>) -> Self {
            Self {
                name:      CallUpdater   .create(updater.name)     .must_be_initialized(),
                job_codes: CallUpdater   .create(updater.job_codes).with_defaulter(Default::default),
                rank:      MoveOwnership .create(updater.rank)     .must_be_initialized(),
                age:       MoveOwnership .create(updater.age)      .with_defaulter(Default::default),
                gender:    (Gender::from_str as fn(&str) -> Gender)
                                         .create(updater.gender)   .must_be_initialized(),
                style:     ReadStyle     .create(updater.style)    .with_defaulter(MyStyle::new),
            }
        }
    }

    impl<T1, T2, T3, T4, T5, T6> AnonymousStruct<T1, T2, T3, T4, T5, T6> {
        pub fn set_name<T>(self, value: T) -> AnonymousStruct<(T,), T2, T3, T4, T5, T6> {
            AnonymousStruct {
                name: (value,),
                job_codes: self.job_codes,
                rank: self.rank,
                age: self.age,
                gender: self.gender,
                style: self.style
            }
        }

        pub fn set_job_codes<T>(self, value: T) -> AnonymousStruct<T1, (U,), T3, T4, T5, T6> {
            AnonymousStruct {
                name: self.name,
                job_codes: (value,),
                rank: self.rank,
                age: self.age,
                gender: self.gender,
                style: self.style
            }
        }
    }
}