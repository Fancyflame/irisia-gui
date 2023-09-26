use proc_macro2::TokenStream;
use quote::quote;
use syn::WhereClause;

struct ListenOptions {
    once: bool,
    trusted: bool,
    asyn: bool,
    sub_event: bool,
    no_handle: bool,
}

impl ListenOptions {
    fn impl_item(&self) -> TokenStream {
        let arg_types = [
            self.once,
            self.trusted,
            self.asyn,
            self.sub_event,
            self.no_handle,
        ]
        .into_iter()
        .map(|on| if on { quote!(FlagSet) } else { quote!(()) });

        let fut_generic = if self.asyn { Some(quote!(Ret,)) } else { None };

        let where_clause = self.where_clause();
        let fn_body = self.fn_body();

        quote! {
            impl<Ep> Listen<'_, Ep, #(#arg_types),*> {
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

        let spwan_task = match (self.asyn, self.no_handle) {
            (false, false) => {
                quote!(f(result, &obj))
            }
            (false, true) => {
                quote!(f(result))
            }
            (true, false) => {
                quote!(tokio::task::spawn_local(f(result, obj.clone())))
            }
            (true, true) => {
                quote!(tokio::task::spawn_local(f(result)))
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
            let obj = self.ep.clone();
            self.ep.daemon(async move {
                #future
            })
        }
    }

    fn where_clause(&self) -> WhereClause {
        let mut tokens = quote! {
            where
                Ep: EdProvider,
        };

        let e_bound = if self.sub_event {
            quote!(SubEvent)
        } else {
            quote!(Event)
        };
        tokens.extend(quote!(E: #e_bound,));

        if self.asyn {
            tokens.extend(quote!(Ret: Future<Output = ()> + 'static,));
        }

        let f_bound = {
            let handle_arg = match (self.no_handle, self.asyn) {
                (true, _) => quote!(),
                (false, false) => quote!(&Ep),
                (false, true) => quote!(Ep),
            };
            match (self.once, self.asyn) {
                (false, false) => quote!(FnMut(E, #handle_arg)),
                (true, false) => quote!(FnOnce(E, #handle_arg)),
                (false, true) => quote!(FnMut(E, #handle_arg) -> Ret),
                (true, true) => quote!(FnOnce(E, #handle_arg) -> Ret),
            }
        };
        tokens.extend(quote!(F: #f_bound + 'static,));

        syn::parse2(tokens).unwrap()
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
            no_handle: index & 0b10000 != 0,
        };

        if options.sub_event && options.trusted {
            continue;
        }

        tokens.extend(options.impl_item());
    }

    tokens
}
