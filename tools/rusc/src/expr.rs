use syn::{Result, Ident, Token, token, punctuated::Punctuated, parse::{Parse, ParseStream}};

pub enum Expr {
    Call(Call),
    MethodCall(MethodCall)
}
impl Parse for Expr {
    fn parse(input: ParseStream) -> Result<Self> {
        
    }
}
struct Call {
    dot_token: Token![.],
    func: Ident,
    paren_token: token::Paren,
    args: Punctuated<Expr, Token![,]>
}
struct MethodCall {
    receiver: Box<Expr>,
    dot_token: Token![.],
    method: Ident,
    paren_token: token::Paren,
    args: Punctuated<Expr, Token![,]>
}
