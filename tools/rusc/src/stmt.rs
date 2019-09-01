use syn::{Result, Token, Ident, token, parse::{Parse, ParseStream}};

use crate::expr::Expr;

pub enum Stmt {
    Js(Js),
    Semi(Expr, Token![;]),
    Expr(Expr)
}
impl Parse for Stmt {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        loop {
            if input.peek(token::Paren) {
                let content;
                Ok(Stmt::())
            }
        }
    }
}

struct Block {
    members: Vec<Member>,
    fns: Vec<Fn>
}