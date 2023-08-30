use proc_macro2::TokenStream;
use quote::quote;
use syn::WhereClause;

struct ListenOptions {
    once: bool,
    sys_only: bool,
    asyn: bool,
    sub_event: bool,
    without_handle: bool,
}

impl ListenOptions {
    fn impl_item(&self) -> TokenStream {
        let arg_types = [
            self.once,
            self.sys_only,
            self.asyn,
            self.sub_event,
            self.without_handle,
        ]
        .into_iter()
        .map(|on| if on { quote!(FlagSet) } else { quote!(()) });

        let fut_generic = if self.asyn { Some(quote!(Ret,)) } else { None };

        let where_clause = self.where_clause();
        let fn_body = self.fn_body();

        let impl_head = if self.without_handle {
            quote!(impl<Eh> Listen<Eh, #(#arg_types),*>)
        } else {
            quote!(impl<Eh> Listen<&Eh, #(#arg_types),*>)
        };

        quote! {
            #impl_head {
                pub fn spawn<E, F, #fut_generic>(self, mut f: F) -> JoinHandle<()>
                #where_clause
                { #fn_body }
            }
        }
    }

    fn fn_body(&self) -> TokenStream {
        let (cloned_obj, call_ed, callback) = if self.without_handle {
            (quote!(self.eh.ed.clone()), quote!(obj), quote!(f(result)))
        } else {
            (
                quote!(self.eh.clone()),
                quote!(obj.ed),
                quote!(f(result, &obj)),
            )
        };

        let recv_method = match (self.sub_event, self.sys_only) {
            (false, false) => quote!(receiver.recv().await.0),
            (false, true) => quote!(receiver.recv_sys().await),
            (true, false) => quote!(E::handle(&mut receiver).await),
            (true, true) => panic!("sub event can never be system event"),
        };

        let spwan_task = if self.asyn {
            quote!(tokio::spawn(#callback))
        } else {
            callback
        };

        let future = if self.once {
            quote! {
                let mut receiver = EventReceiver::EventDispatcher(&#call_ed);
                let result = #recv_method;
                #spwan_task;
            }
        } else {
            quote! {
                let mut receiver = EventReceiver::Lock(#call_ed.lock());
                loop {
                    let result = #recv_method;
                    #spwan_task;
                }
            }
        };

        quote! {
            let obj = #cloned_obj;
            tokio::spawn(async move {
                #call_ed.cancel_on_abandoned(async {
                    #future
                }).await;
            })
        }
    }

    fn where_clause(&self) -> WhereClause {
        let mut tokens = quote!(where);

        let (handle_extra_bound, handle_arg) = if self.without_handle {
            (quote!(), quote!())
        } else {
            (quote!(+ Clone + Send + Sync + 'static), quote!(&Eh,))
        };

        tokens.extend(quote!(Eh: Deref<Target = ElementHandle> #handle_extra_bound,));

        let e_bound = if self.sub_event {
            quote!(SubEvent)
        } else {
            quote!(Event)
        };
        tokens.extend(quote!(E: #e_bound,));

        if self.asyn {
            tokens.extend(quote!(Ret: Future<Output = ()> + Send + 'static,));
        }

        let f_bound = match (self.once, self.asyn) {
            (false, false) => quote!(FnMut(E, #handle_arg)),
            (true, false) => quote!(FnOnce(E, #handle_arg)),
            (false, true) => quote!(FnMut(E, #handle_arg) -> Ret),
            (true, true) => quote!(FnOnce(E, #handle_arg) -> Ret),
        };
        tokens.extend(quote!(F: #f_bound + Send + 'static,));

        syn::parse2(tokens).unwrap()
    }
}

pub fn impl_listen() -> TokenStream {
    let mut tokens = TokenStream::new();

    for index in 0b00000..=0b11111 {
        let options = ListenOptions {
            once: index & 0b00001 != 0,
            sys_only: index & 0b00010 != 0,
            asyn: index & 0b00100 != 0,
            sub_event: index & 0b01000 != 0,
            without_handle: index & 0b10000 != 0,
        };

        if options.sub_event && options.sys_only {
            continue;
        }

        tokens.extend(options.impl_item());
    }

    tokens
}
