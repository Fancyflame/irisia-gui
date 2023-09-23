use proc_macro2::TokenStream;
use quote::quote;
use syn::WhereClause;

struct ListenOptions {
    once: bool,
    sys_only: bool,
    asyn: bool,
    sub_event: bool,
}

fn rc_em() -> TokenStream {
    quote!(Rc<ElementModel<El, Sty, Sc>>)
}

impl ListenOptions {
    fn impl_item(&self) -> TokenStream {
        let arg_types = [self.once, self.sys_only, self.asyn, self.sub_event]
            .into_iter()
            .map(|on| if on { quote!(FlagSet) } else { quote!(()) });

        let fut_generic = if self.asyn { Some(quote!(Ret,)) } else { None };

        let where_clause = self.where_clause();
        let fn_body = self.fn_body();

        let em = rc_em();
        quote! {
            impl<El, Sty, Sc> Listen<&#em, #(#arg_types),*>
            where
                El: Element,
                Sty: StyleContainer + 'static,
                Sc: RenderMultiple + 'static,
            {
                pub fn spawn<E, F, #fut_generic>(self, mut f: F) -> JoinHandle<()>
                #where_clause
                { #fn_body }
            }
        }
    }

    fn fn_body(&self) -> TokenStream {
        let recv_method = match (self.sub_event, self.sys_only) {
            (false, false) => quote!(receiver.recv().await.0),
            (false, true) => quote!(receiver.recv_sys().await),
            (true, false) => quote!(E::handle(&mut receiver).await),
            (true, true) => panic!("sub event can never be system event"),
        };

        let spwan_task = if self.asyn {
            quote!(tokio::task::spawn_local(f(result, obj.clone())))
        } else {
            quote!(f(result, &obj))
        };

        let future = if self.once {
            quote! {
                let mut receiver = EventReceiver::EventDispatcher(&obj.ed);
                let result = #recv_method;
                if !obj.alive() {
                    return;
                }
                #spwan_task;
            }
        } else {
            quote! {
                let mut receiver = EventReceiver::Lock(obj.ed.lock());
                loop {
                    let result = #recv_method;
                    if !obj.alive() {
                        return;
                    }
                    #spwan_task;
                }
            }
        };

        quote! {
            let obj = self.eh.clone();
            tokio::task::spawn_local(async move {
                obj.ed.cancel_on_abandoned(async {
                    #future
                }).await;
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

        let element_model = rc_em();

        let f_bound = match (self.once, self.asyn) {
            (false, false) => quote!(FnMut(E, &#element_model)),
            (true, false) => quote!(FnOnce(E, &#element_model)),
            (false, true) => quote!(FnMut(E, #element_model) -> Ret),
            (true, true) => quote!(FnOnce(E, #element_model) -> Ret),
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
            sys_only: index & 0b00010 != 0,
            asyn: index & 0b00100 != 0,
            sub_event: index & 0b01000 != 0,
        };

        if options.sub_event && options.sys_only {
            continue;
        }

        tokens.extend(options.impl_item());
    }

    tokens
}
