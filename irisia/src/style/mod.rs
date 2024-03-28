use impl_variadics::impl_variadics;

pub mod anima;

#[derive(Clone, Copy)]
pub enum StyleValue {
    Delimiter,
    Float(f32),
    Ident(&'static str),
    Color([u8; 4]),
}

pub trait StyleSource {
    fn get_style<'a>(&'a self, name: &str, prog: f32) -> Option<&'a [StyleValue]>;
}

pub trait TupleStyleGroup: StyleSource {
    type Chained<T: StyleSource>: TupleStyleGroup;

    fn chain<T>(self, other: T) -> Self::Chained<T>
    where
        Self: Sized,
        T: StyleSource;
}

type Pair<T> = (&'static str, T);

impl<T> StyleSource for Pair<T>
where
    T: AsRef<[StyleValue]>,
{
    fn get_style<'a>(&'a self, name: &str, _: f32) -> Option<&'a [StyleValue]> {
        if name == self.0 {
            Some(self.1.as_ref())
        } else {
            None
        }
    }
}

impl_variadics! {
    ..30 "Sty*" => {
        impl<#(#Sty0,)*> TupleStyleGroup for (#(#Sty0,)*)
        where
            #(#Sty0: StyleSource,)*
        {
            type Chained<T: StyleSource> = (T, #(#Sty0,)*);

            fn chain<T>(self, other: T) -> Self::Chained<T>
            where
                Self: Sized,
                T: StyleSource
            {
                (
                    other,
                    #(self.#index,)*
                )
            }
        }
    };

    30..31 "Sty*" "style*" => {
        impl<#(#Sty0,)*> TupleStyleGroup for (#(#Sty0,)*)
        where
            #(#Sty0: StyleSource,)*
        {
            type Chained<T: StyleSource> = ((T,), #(#Sty0,)*);

            fn chain<T>(self, other: T) -> Self::Chained<T>
            where
                Self: Sized,
                T: StyleSource
            {
                (
                    (other,),
                    #(self.#index,)*
                )
            }
        }

        impl<Fst, #(#Sty0,)*> TupleStyleGroup for (Fst, #(#Sty0,)*)
        where
            Fst: TupleStyleGroup,
            #(#Sty0: StyleSource,)*
        {
            type Chained<T: StyleSource> = (Fst::Chained<T>, #(#Sty0,)*);

            fn chain<T>(self, other: T) -> Self::Chained<T>
            where
                Self: Sized,
                T: StyleSource
            {
                let (first, #(#style0,)*) = self;
                (first.chain(other), #(#style0,)*)
            }
        }
    };

    ..32 "Sty*" => {
        impl<#(#Sty0,)*> StyleSource for (#(#Sty0,)*)
        where
            #(#Sty0: StyleSource,)*
        {
            fn get_style<'a>(&'a self, _name: &str, _prog: f32) -> Option<&'a [StyleValue]> {
                #(if let arr @ Some(_) = self.#index.get_style(_name, _prog) {
                    arr
                } else)* {
                    None
                }
            }
        }
    }
}
