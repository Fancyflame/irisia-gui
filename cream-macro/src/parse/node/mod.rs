use proc_macro2::Span;
use std::collections::{HashMap, HashSet};
use syn::{
    braced, parenthesized,
    parse::{Parse, ParseBuffer, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Brace, Paren},
    Expr, Ident, LitInt, Token, TypePath,
};

use self::{style::{StyleItem, Style}, listen::ListenList};

pub mod style;
pub mod listen;

pub enum Children {
    None,
    Single(Box<Node>),
    Multiple(Vec<Node>),
}

pub struct Node {
    pub span: Span,
    pub node_type: TypePath,
    pub expr_queue: Vec<Expr>,
    pub required_args: Vec<usize>,
    pub optional_args: HashMap<Ident, usize>,
    pub id_rename: Option<Ident>,
    pub styles: Vec<StyleItem>,
    pub listen_list:ListenList,
    pub children: Children,
}

enum NodeBody {
    RequiredArg(usize, Expr),
    OptionalArg(Ident, Expr),
    Command(Command),
    Child(Node),
}

struct Command {
    span: Span,
    name: String,
    body: CommandBody,
}

enum CommandBody {
    Id(Ident),
    Style(Style),
    ListenList(ListenList)
}

impl Parse for Node {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // 解析节点名称
        let node_type: TypePath = input.parse()?;

        // HashMap<第N项, 对应的Expr表位置>
        let mut required_args: HashMap<usize, usize> = HashMap::new();

        // 尝试解析圆括号内的必须参数
        // short_req_arg：已解析的圆括号参数数量
        // expr_queue：表达式列表。用于以index来引用。
        let (short_req_arg, mut expr_queue): (Option<usize>, Vec<Expr>) = if input.peek(Paren) {
            let content: ParseBuffer;
            parenthesized!(content in input);

            let queue: Vec<Expr> = content
                .parse_terminated::<_, Token![,]>(Expr::parse)?
                .into_iter()
                .collect();
            (Some(queue.len()), queue)
        } else {
            (None, Vec::new())
        };

        // HashMap<可选参数名称, 对应表达式下标>
        let mut optional_args: HashMap<Ident, usize> = HashMap::new();
        // 子节点列表
        let mut children: Vec<Node> = Vec::new();
        // 样式表
        let mut styles: Vec<StyleItem> = Vec::new();
        // 监听事件
        let mut listen_list:ListenList=ListenList::new();
        // 重命名在最终结构体里的域名
        let mut id_rename = None;

        // 防止使用重复命令
        let mut cmd_hash = HashSet::new();

        // Parse the body of the node.
        if input.peek(Brace) {
            let content: ParseBuffer;
            braced!(content in input);

            let punct: Punctuated<NodeBody, Token![,]> = content.parse_terminated(NodeBody::parse)?;

            for pair in punct.into_pairs() {
                match pair.into_value() {
                    NodeBody::RequiredArg(index, ex) => {
                        if short_req_arg.is_some() {
                            parse_err!(
                                ex.span(), 
                                "Cannot use both parentheses syntax and index syntax to identify required arguments in the same node."
                            );
                        }
                        // Index to the end of the queue
                        required_args.insert(index, expr_queue.len());
                        expr_queue.push(ex);
                    }
                    NodeBody::OptionalArg(k, v) => {
                        optional_args.insert(k, expr_queue.len());
                        expr_queue.push(v);
                    }
                    NodeBody::Child(node) => {
                        children.push(node);
                    }
                    NodeBody::Command(cmd) => {
                        if cmd_hash.contains(&cmd.name) {
                            parse_err!(cmd.span, "Command `{}` is duplicated", cmd.name);
                        }else{
                            cmd_hash.insert(cmd.name);
                        }

                        match cmd.body {
                            CommandBody::Id(name) => id_rename = Some(name),
                            CommandBody::Style(vec) => styles=vec.0,
                            CommandBody::ListenList(ll)=>listen_list=ll,
                        }
                    }
                }
            }
        }

        let required_args = match short_req_arg {
            Some(span) => Vec::from_iter(0..span),
            None => {
                let mut vec = Vec::with_capacity(required_args.len());
                for x in 0..required_args.len() {
                    match required_args.get(&x){
                        Some(n)=>vec.push(*n),
                        None=>{
                            parse_err!(node_type.span(),"Index `{x}` of required arguments list is missing");
                        }
                    }
                }
                vec
            }
        };

        Ok(Node {
            span: node_type.span(),
            node_type,
            expr_queue,
            required_args,
            optional_args,
            id_rename,
            styles,
            listen_list,
            children: match children.len() {
                0 => Children::None,
                1 => Children::Single(Box::new(children.pop().unwrap())),
                _ => Children::Multiple(children),
            }
        })
    }
}

impl Parse for NodeBody {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(LitInt) {
            // required arguments
            let int: LitInt = input.parse()?;
            input.parse::<Token![:]>()?;
            Ok(NodeBody::RequiredArg(int.base10_parse()?, input.parse()?))
        } else if input.peek(Ident) && input.peek2(Token![:]) {
            // optional arguments
            let field = input.parse()?;
            input.parse::<Token![:]>()?;
            let value: Expr = input.parse()?;
            Ok(NodeBody::OptionalArg(field, value))
        } else if input.peek(Token![:]) {
            // command
            Ok(NodeBody::Command(input.parse()?))
        } else {
            // node
            Ok(NodeBody::Child(input.parse()?))
        }
    }
}

impl Parse for Command {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![:]>()?;
        let id = input.parse::<Ident>()?;
        let id_str=id.to_string();
        input.parse::<Token![:]>()?;

        let body=match &*id_str {
            "id" => CommandBody::Id(input.parse()?),
            "style" => CommandBody::Style(input.parse()?),
            "on"=>CommandBody::ListenList(input.parse()?),
            other => parse_err!(id.span(), "Unrecognized command: `{}`", other),
        };

        Ok(Command{
            span:id.span(),
            name:id_str,
            body
        })
    }
}