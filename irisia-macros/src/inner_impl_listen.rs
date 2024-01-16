use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse_quote, punctuated::Punctuated, Token, WhereClause};

struct ListenOptions {
    once: bool,
    trusted: bool,
    asyn: bool,
    sub_event: bool,
    with_handle: bool,
}

impl ListenOptions {
    fn impl_item(&self) -> TokenStream {
        let arg_types: Vec<TokenStream> = [
            self.once,
            self.trusted,
            self.asyn,
            self.sub_event,
            self.with_handle,
        ]
        .into_iter()
        .map(|on| if on { quote!(FlagSet) } else { quote!(()) })
        .collect();

        let (where_clause, generics) = self.where_clause();
        let fn_body = self.fn_body();

        /*quote! {
            impl<Ep> Listen<'_, Ep, #(#arg_types),*> {
                pub fn spawn<E, F, #fut_generic>(self, mut f: F) -> JoinHandle<()>
                #where_clause
                { #fn_body }
            }

            #auto_infer
        }*/

        quote! {
            impl<#(#generics,)*> ListenerOption<Ep, E, #(#arg_types,)*> for F
            #where_clause
            {
                fn listen_from(mut self, l: Listen<Ep, #(#arg_types,)*>) -> JoinHandle<()>
                { #fn_body }
            }
        }
    }

    fn fn_body(&self) -> TokenStream {
        let recv_method = match (self.sub_event, self.trusted) {
            (false, false) => quote!(receiver.recv().await.0),
            (false, true) => quote!(receiver.recv_trusted().await),
            (true, false) => quote!(E::handle(&mut receiver).await),
            (true, true) => unreachable!("sub event can never be trusted event"),
        };

        let spwan_task = match (self.asyn, self.with_handle) {
            (false, false) => {
                quote!(self(result))
            }
            (false, true) => {
                quote!(self(result, &obj))
            }
            (true, false) => {
                quote!(tokio::task::spawn_local(self(result)))
            }
            (true, true) => {
                quote!(tokio::task::spawn_local(self(result, obj.clone())))
            }
        };

        let future = if self.once {
            quote! {
                let mut receiver = EventReceiver::EventDispatcher(obj.event_dispatcher());
                let result = #recv_method;
                if !obj.handle_available() {
                    return;
                }
                #spwan_task;
            }
        } else {
            quote! {
                let mut receiver = EventReceiver::Lock(obj.event_dispatcher().lock());
                loop {
                    let result = #recv_method;
                    if !obj.handle_available() {
                        return;
                    }
                    #spwan_task;
                }
            }
        };

        quote! {
            let obj = l.ep.clone();
            l.ep.daemon(async move {
                #future
            })
        }
    }

    // (_, generics)
    fn where_clause(&self) -> (WhereClause, Vec<Ident>) {
        let mut tokens = quote! {
            where
                Ep: EdProvider,
        };

        let mut generics: Vec<Ident> = vec![parse_quote!(F), parse_quote!(Ep), parse_quote!(E)];

        let e_bound = if self.sub_event {
            quote!(SubEvent)
        } else {
            quote!(Event)
        };
        tokens.extend(quote!(E: #e_bound,));

        if self.asyn {
            tokens.extend(quote!(Ret: Future<Output = ()> + 'static,));
            generics.push(parse_quote!(Ret));
        }

        let f_bound = {
            let handle_arg = match (self.with_handle, self.asyn) {
                (false, _) => quote!(),
                (true, false) => quote!(&Ep),
                (true, true) => quote!(Ep),
            };

            let fn_type = if self.once {
                quote!(FnOnce)
            } else {
                quote!(FnMut)
            };

            let ret = if self.asyn {
                Some(quote!(-> Ret))
            } else {
                None
            };

            quote!(#fn_type(E, #handle_arg) #ret)
        };

        tokens.extend(quote!(F: #f_bound + 'static,));

        (syn::parse2(tokens).unwrap(), generics)
    }

    fn auto_infer(&self, where_clause: &WhereClause) -> TokenStream {
        let Self {
            asyn,
            sub_event,
            with_handle,
            ..
        } = self;

        let mut generics: Punctuated<TokenStream, Token![,]> = Punctuated::new();
        generics.push(quote!(F));
        generics.push(quote!(E));
        if *asyn {
            generics.push(quote!(Ret));
        }
        if *with_handle {
            generics.push(quote!(Ep));
        }

        let (flag_set, flag_unset) = (quote!(FlagSet), quote!(()));

        let flags = [asyn, sub_event, with_handle].into_iter().map(|on| {
            if *on {
                &flag_set
            } else {
                &flag_unset
            }
        });

        let marker = generics.iter().skip(1);

        quote! {
            impl<#generics> ListenerOption<#(#flags,)* (#(#marker,)*)> for F
            #where_clause
            {}
        }
    }
}

pub fn impl_listen() -> TokenStream {
    let mut tokens = TokenStream::new();

    for index in 0..=0b11111 {
        let options = ListenOptions {
            once: index & 0b00001 != 0,
            trusted: index & 0b00010 != 0,
            asyn: index & 0b00100 != 0,
            sub_event: index & 0b01000 != 0,
            with_handle: index & 0b10000 != 0,
        };

        if options.sub_event && options.trusted {
            continue;
        }

        tokens.extend(options.impl_item());
    }

    tokens
}
