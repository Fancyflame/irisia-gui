use super::{conditional::StateConditional, *};

pub enum StateExpr<T>
where
    T: StmtTree,
{
    Raw(T::Stmt),
    Block(StateBlock<T>),
    Conditional(StateConditional<T>),
    Repetitive(StateRepetitive<T>),
    Command(StateCommand<T>),
}

impl<T: StmtTree> Parse for StateExpr<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let r = if input.peek(Token![if]) {
            StateExpr::Conditional(StateConditional::If(input.parse()?))
        } else if input.peek(Token![match]) {
            StateExpr::Conditional(StateConditional::Match(input.parse()?))
        } else if input.peek(Token![while]) {
            StateExpr::Repetitive(StateRepetitive::While(input.parse()?))
        } else if input.peek(Token![for]) {
            StateExpr::Repetitive(StateRepetitive::For(input.parse()?))
        } else if input.peek(Brace) {
            StateExpr::Block(input.parse()?)
        } else if input.peek(Token!(@)) {
            StateExpr::Command(input.parse()?)
        } else {
            StateExpr::Raw(input.parse()?)
        };
        Ok(r)
    }
}

macro_rules! impl_to_tokens {
    ($($Arm:ident)*) => {
        impl<T> ToTokens for StateExpr<T>
        where
            T: StmtTree + StmtTreeCodegen,
            T::Stmt: ToTokens,
         {
            fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
                match self {
                    $(StateExpr::$Arm(x) => x.to_tokens(tokens),)*
                }
            }
        }
    }
}

impl_to_tokens!(Raw Block Conditional Repetitive Command);

macro_rules! impl_from {
    ($($Arm:ident $Type:ident,)*) => {
        $(
            impl<T: StmtTree> From<$Type<T>> for StateExpr<T> {
                fn from(e: $Type<T>) -> Self {
                    Self::$Arm(e)
                }
            }
        )*
    };
}

impl_from! {
    Block StateBlock,
    Conditional StateConditional,
    Repetitive StateRepetitive,
    Command StateCommand,
}
