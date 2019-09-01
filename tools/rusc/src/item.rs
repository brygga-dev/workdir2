use syn::{Result, Token, Ident, token, parenthesized, punctuated::Punctuated, parse::{Parse, ParseStream}};

use crate::stmt::Stmt;

pub enum Item {
    Struct(Struct),
    Fn(Fn),
    Stmt(Stmt)
}
impl Parse for Item {
    fn parse(input: ParseStream) -> Result<Self> {
        let ahead = input.lookahead1();
        if ahead.peek(Token![struct]) {
            Ok(Item::Struct(input.parse()?))
        } else if ahead.peek(Token![fn]) {
            Ok(Item::Fn(input.parse()?))
        } else {
            // Statement
            Ok(Item::Stmt(input.parse()?))
        }
    }
}

#[derive(Debug)]
struct Fn {
    fn_token: Token![fn],
    ident: Ident,
    parens: token::Paren,
    args: Punctuated<Arg, Token![,]>,
    block: syn::Block
}
impl Parse for Fn {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Fn {
            fn_token: input.parse()?,
            ident: input.parse()?,
            parens: parenthesized!(content in input),
            args: content.parse_terminated(Arg::parse)?,
            block: input.parse()?
        })
    }
}
#[derive(Debug)]
struct Arg {
    ident: Ident,
    colon: Token![:],
    ty: syn::Type
}
impl Parse for Arg {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Arg {
            ident: input.parse()?,
            colon: input.parse()?,
            ty: input.parse()?
        })
    }
}

struct Member {
    ident: Ident,
    colon: Token![:],
    ty: syn::Type
}

enum StructItem {
    Member(Member),
    Fn(Fn)
}

struct Struct {
    ident: Ident,
    braces: token::Brace,
    items: Vec<StructItem>
}
impl Parse for Struct {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        let content;
        let braces: token::Brace = parenthesized!(content in input);
        let mut members = Vec::new();
        loop {
            if content.peek(Token![fn]) {

            }
        }
        Ok(Struct {
            ident,
            braces,
            members
        })
    }
}