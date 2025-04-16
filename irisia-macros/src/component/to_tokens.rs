use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{Expr, Token};

use crate::component::{Component, FieldAssignMethod, FieldAssignment, ForStmt, IfStmt, Stmt};

use super::{BuildMacro, MatchArm, MatchStmt, WhileStmt};

const_quote! {
    const PATH_CONTROL_FLOW = { irisia::model::control_flow };
    const PATH_COMPONENT = { irisia::model::component };
    const PATH_RC = { ::std::rc::Rc };
    const PATH_OPTION = { ::core::option::Option };
}

impl BuildMacro {
    pub fn gen_code(&self) -> TokenStream {
        gen_rc_chained(&self.0)
    }
}

impl Stmt {
    pub fn gen_code(&self) -> TokenStream {
        match self {
            Stmt::Block(block) => gen_chained(&block.stmts),
            Stmt::For(for_stmt) => gen_for(for_stmt),
            Stmt::While(while_stmt) => gen_while(while_stmt),
            Stmt::If(if_stmt) => gen_if(if_stmt),
            Stmt::Match(match_stmt) => gen_match(match_stmt),
            Stmt::Component(comp) => gen_component(comp),
            Stmt::UseExpr(expr) => expr.clone(),
        }
    }
}

fn gen_rc_chained(stmts: &[Stmt]) -> TokenStream {
    let chained = gen_chained(stmts);
    quote! {
        #PATH_RC::new(#chained)
    }
}

fn gen_chained(stmts: &[Stmt]) -> TokenStream {
    match stmts {
        [] => quote! {()},
        [one] => one.gen_code(),
        _ => {
            let (p1, p2) = stmts.split_at(stmts.len() / 2);
            let p1 = gen_chained(p1);
            let p2 = gen_chained(p2);
            quote! {
                (#p1, #p2)
            }
        }
    }
}

fn gen_for(
    ForStmt {
        pattern,
        expr,
        body,
        get_key,
    }: &ForStmt,
) -> TokenStream {
    let body = gen_chained(&body.stmts);
    quote! {
        ::std::vec::Vec::from_iter(
            ::core::iter::Iterator::map(
                ::core::iter::IntoIterator::into_iter(#expr),
                |#pattern| (#get_key, #body)
            )
        )
    }
}

fn gen_while(WhileStmt { condition, body }: &WhileStmt) -> TokenStream {
    let body = gen_chained(&body.stmts);
    quote! {
        ::std::vec::Vec::from_iter(
            ::core::iter::from_fn(|| {
                if #condition {
                    #PATH_OPTION::Some(#body)
                } else {
                    #PATH_OPTION::None
                }
            })
        )
    }
}

fn gen_if(
    IfStmt {
        condition,
        then_branch,
        else_branch,
    }: &IfStmt,
) -> TokenStream {
    let then_branch = gen_chained(&then_branch.stmts);
    let else_branch = match else_branch {
        Some(b) => b.gen_code(),
        None => quote! {()},
    };

    quote! {
        if #condition {
            #PATH_CONTROL_FLOW::branch::branch_a(
                #then_branch
            )
        } else {
            #PATH_CONTROL_FLOW::branch::branch_b(
                #else_branch
            )
        }
    }
}

fn gen_match(MatchStmt { expr, arms }: &MatchStmt) -> TokenStream {
    let body = match_arm_binary_fold(&arms, &mut Vec::new());
    quote! {
        match #expr {
            #body
        }
    }
}

fn match_arm_binary_fold(slice: &[MatchArm], branch_stack: &mut Vec<bool>) -> TokenStream {
    let MatchArm {
        pattern,
        guard,
        body,
    } = match slice {
        [] => return quote! {},
        [arm] => arm,
        _ => {
            let (a, b) = slice.split_at(slice.len() / 2);

            branch_stack.push(true);
            let a = match_arm_binary_fold(a, branch_stack);
            branch_stack.pop();

            branch_stack.push(false);
            let b = match_arm_binary_fold(b, branch_stack);
            branch_stack.pop();

            return quote! {#a #b};
        }
    };

    let mut arm_value = body.gen_code();
    for &branch_is_a in branch_stack.iter().rev() {
        arm_value = if branch_is_a {
            quote! {
                #PATH_CONTROL_FLOW::branch::branch_a(#arm_value)
            }
        } else {
            quote! {
                #PATH_CONTROL_FLOW::branch::branch_b(#arm_value)
            }
        }
    }

    let if_token = guard.is_some().then(<Token![if]>::default);
    quote! {
        #pattern #if_token #guard => #arm_value,
    }
}

fn gen_component(
    Component {
        type_path,
        fields,
        body,
    }: &Component,
) -> TokenStream {
    let mut fields: Vec<&FieldAssignment> = fields.iter().collect();
    let mut _body_fa = None;
    if !body.is_empty() {
        fields.push(_body_fa.insert(FieldAssignment {
            name: format_ident!("children"),
            method: FieldAssignMethod::HostingSignal,
            value: Expr::Verbatim(gen_rc_chained(&body)),
        }));
    };

    let defs = field_asgn_binary_fold(&fields, &|fa| {
        let FieldAssignment {
            name: _,
            value,
            method,
        } = fa;

        match method {
            FieldAssignMethod::HostingSignal => {
                quote! {
                    #PATH_COMPONENT::proxy_signal_helper::check_eq(#value).get()
                }
            }
            FieldAssignMethod::Direct => {
                quote! {
                    #PATH_COMPONENT::definition::DirectAssign(#value)
                }
            }
        }
    });

    let names_tuple = field_asgn_binary_fold(&fields, &|fa| fa.name.to_token_stream());
    let assignments = fields.iter().map(|fa| {
        let name = &fa.name;
        let value = match fa.method {
            FieldAssignMethod::HostingSignal => {
                quote! {
                    irisia::coerce_hook!(#name)
                }
            }
            FieldAssignMethod::Direct => {
                quote! { #name }
            }
        };

        quote! {
            #name: #PATH_OPTION::Some(#value),
        }
    });

    let create_fn = quote! {
        |#names_tuple| {
            #type_path {
                #(#assignments)*
                ..::core::default::Default::default()
            }
        }
    };

    quote! {
        {
            #PATH_COMPONENT::UseComponent::<#type_path, _, _>::new(
                #create_fn,
                #defs,
            )
        }
    }
}

fn field_asgn_binary_fold<F>(slice: &[&FieldAssignment], for_each: &F) -> TokenStream
where
    F: Fn(&FieldAssignment) -> TokenStream,
{
    match slice {
        [] => quote! {()},
        [one] => for_each(one),
        _ => {
            let (a, b) = slice.split_at(slice.len() / 2);
            let a = field_asgn_binary_fold(a, for_each);
            let b = field_asgn_binary_fold(b, for_each);
            quote! {(#a, #b)}
        }
    }
}
