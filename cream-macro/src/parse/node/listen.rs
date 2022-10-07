use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{Brace, Bracket},
    Ident, Type,
};

/*
{
    Event1 => listener1,
    Event2 => listener2,
    Event3 => [listener3, listener4]
}

Or

Event1 => [listener1, listener2]
*/
pub struct ListenList(pub Vec<Listen>);

pub struct Listen {
    pub event: Type,
    pub listeners: Vec<Ident>,
}

impl ListenList {
    pub fn new() -> Self {
        ListenList(Vec::new())
    }
}

impl Parse for ListenList {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        braced!(content in input);

        let parse_item = |ps: ParseStream| {
            let event = ps.parse()?;
            ps.parse::<Token!(=>)>()?;

            let listeners = if ps.peek(Bracket) {
                let bracketed;
                bracketed!(bracketed in ps);
                let punct: Punctuated<Ident, Token!(,)> =
                    bracketed.parse_terminated(Ident::parse)?;
                punct.into_iter().collect()
            } else {
                vec![ps.parse()?]
            };

            Ok(Listen { event, listeners })
        };

        let vec = if content.peek(Brace) {
            let braced;
            braced!(braced in content);

            braced
                .parse_terminated::<_, Token!(,)>(parse_item)?
                .into_iter()
                .collect()
        } else {
            vec![parse_item(&content)?]
        };

        Ok(ListenList(vec))
    }
}
