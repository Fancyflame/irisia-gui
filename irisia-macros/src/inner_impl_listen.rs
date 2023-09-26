use proc_macro2::TokenStream;
use quote::quote;
use syn::WhereClause;

struct ListenOptions {
    once: bool,
    trusted: bool,
    asyn: bool,
    sub_event: bool,
}

impl ListenOptions {
    fn impl_item(&self) -> TokenStream {
        let arg_types = [self.once, self.trusted, self.asyn, self.sub_event]
            .into_iter()
            .map(|on| if on { quote!(FlagSet) } else { quote!(()) });

        let fut_generic = if self.asyn { Some(quote!(Ret,)) } else { None };

        let where_clause = self.where_clause();
        let fn_body = self.fn_body();

        quote! {
            impl<Ep> Listen<'_, Ep, #(#arg_types),*>
            where
                Ep: EdProvider
            {
                pub fn spawn<E, F, #fut_generic>(self, mut f: F) -> JoinHandle<()>
                #where_clause
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

        let spwan_task = if self.asyn {
            quote!(tokio::task::spawn_local(f(result, obj.clone())))
        } else {
            quote!(f(result, &obj))
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
            let obj = self.ep.clone();
            self.ep.daemon(async move {
                #future
            })
        }
    }

    fn where_clause(&self) -> WhereClause {
        let mut tokens = quote!(where);

        let e_bound = if self.sub_event {
            quote!(SubEvent)
        } else {
            quote!(Event)
        };
        tokens.extend(quote!(E: #e_bound,));

        if self.asyn {
            tokens.extend(quote!(Ret: Future<Output = ()> + 'static,));
        }

        let f_bound = match (self.once, self.asyn) {
            (false, false) => quote!(FnMut(E, &Ep)),
            (true, false) => quote!(FnOnce(E, &Ep)),
            (false, true) => quote!(FnMut(E, Ep) -> Ret),
            (true, true) => quote!(FnOnce(E, Ep) -> Ret),
        };
        tokens.extend(quote!(F: #f_bound + 'static,));

        syn::parse2(tokens).unwrap()
    }
}

pub fn impl_listen() -> TokenStream {
    let mut tokens = TokenStream::new();

    for index in 0b0000..=0b1111 {
        let options = ListenOptions {
            once: index & 0b00001 != 0,
            trusted: index & 0b00010 != 0,
            asyn: index & 0b00100 != 0,
            sub_event: index & 0b01000 != 0,
        };

        if options.sub_event && options.trusted {
            continue;
        }

        tokens.extend(options.impl_item());
    }

    tokens
}
