use proc_macro2::TokenStream;
use quote::quote;
use syn::Token;

use crate::component::{ForStmt, IfStmt, Stmt};

use super::{BuildMacro, MatchArm, MatchStmt, UseExprStmt, WhileStmt};

mod use_component;

const_quote! {
    const PATH_CONTROL_FLOW = { irisia::model::control_flow };
    const PATH_COMPONENT = { irisia::model::component };
    const PATH_RC = { ::std::rc::Rc };
    const PATH_OPTION = { ::core::option::Option };
}

impl BuildMacro {
    pub fn gen_code(&self) -> TokenStream {
        (GenerationEnv {}).gen_rc_chained(&self.stmts)
    }
}

struct GenerationEnv {}

impl GenerationEnv {
    pub fn gen_code(&self, stmt: &Stmt) -> TokenStream {
        match stmt {
            Stmt::Block(block) => self.gen_chained(&block.stmts),
            Stmt::For(for_stmt) => self.gen_for(for_stmt),
            Stmt::While(while_stmt) => self.gen_while(while_stmt),
            Stmt::If(if_stmt) => self.gen_if(if_stmt),
            Stmt::Match(match_stmt) => self.gen_match(match_stmt),
            Stmt::Component(comp) => self.gen_component(comp),
            Stmt::UseExpr(expr) => self.gen_use_expr(expr),
        }
    }
}

impl GenerationEnv {
    fn gen_rc_chained(&self, stmts: &[Stmt]) -> TokenStream {
        let chained = self.gen_chained(stmts);
        quote! {
            #PATH_RC::new(#chained)
        }
    }

    fn gen_chained(&self, stmts: &[Stmt]) -> TokenStream {
        match stmts {
            [] => self.gen_empty(),
            [one] => self.gen_code(one),
            _ => {
                let (p1, p2) = stmts.split_at(stmts.len() / 2);
                let p1 = self.gen_chained(p1);
                let p2 = self.gen_chained(p2);
                quote! {
                    (#p1, #p2)
                }
            }
        }
    }

    fn gen_for(
        &self,
        ForStmt {
            pattern,
            expr,
            body,
            get_key,
        }: &ForStmt,
    ) -> TokenStream {
        let body = self.gen_chained(&body.stmts);
        quote! {
            ::std::vec::Vec::from_iter(
                ::core::iter::Iterator::map(
                    ::core::iter::IntoIterator::into_iter(#expr),
                    |#pattern| (#get_key, #body)
                )
            )
        }
    }

    fn gen_while(&self, WhileStmt { condition, body }: &WhileStmt) -> TokenStream {
        let body = self.gen_chained(&body.stmts);
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
        &self,
        IfStmt {
            condition,
            then_branch,
            else_branch,
        }: &IfStmt,
    ) -> TokenStream {
        let then_branch = self.gen_chained(&then_branch.stmts);
        let else_branch = match else_branch {
            Some(b) => self.gen_code(&b),
            None => self.gen_empty(),
        };

        quote! {
            if #condition {
                #PATH_CONTROL_FLOW::branch::Branch::A(
                    #then_branch
                )
            } else {
                #PATH_CONTROL_FLOW::branch::Branch::B(
                    #else_branch
                )
            }
        }
    }

    fn gen_match(&self, MatchStmt { expr, arms }: &MatchStmt) -> TokenStream {
        let body = self.match_arm_binary_fold(&arms, &mut Vec::new());
        quote! {
            match #expr {
                #body
            }
        }
    }

    fn match_arm_binary_fold(
        &self,
        slice: &[MatchArm],
        branch_stack: &mut Vec<bool>,
    ) -> TokenStream {
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
                let a = self.match_arm_binary_fold(a, branch_stack);
                branch_stack.pop();

                branch_stack.push(false);
                let b = self.match_arm_binary_fold(b, branch_stack);
                branch_stack.pop();

                return quote! {#a #b};
            }
        };

        let mut arm_value = self.gen_code(body);
        for &branch_is_a in branch_stack.iter().rev() {
            arm_value = if branch_is_a {
                quote! {
                    #PATH_CONTROL_FLOW::branch::Branch::A(#arm_value)
                }
            } else {
                quote! {
                    #PATH_CONTROL_FLOW::branch::Branch::B(#arm_value)
                }
            }
        }

        let if_token = guard.is_some().then(<Token![if]>::default);
        quote! {
            #pattern #if_token #guard => #arm_value,
        }
    }

    fn gen_use_expr(&self, UseExprStmt { value }: &UseExprStmt) -> TokenStream {
        match value {
            Some(value) => value.clone(),
            None => self.gen_empty(),
        }
    }

    fn gen_empty(&self) -> TokenStream {
        quote! {
            ()
        }
    }
}
