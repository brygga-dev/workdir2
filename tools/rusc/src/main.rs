mod file;
mod item;
mod stmt;
mod expr;

use std::path::{Path};
extern crate proc_macro2;
use proc_macro2::TokenStream;

use syn::{Result, Token};
use syn::Ident;
use syn::token;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parenthesized;
use syn::punctuated::Punctuated;



struct Js {
    stmts: Vec<JsStmt>
}
enum JsStmt {
    Semi(JsExpr, Token![;])
}
enum JsExpr {
    MethodCall(JsMethodCall)
}
struct JsMethodCall {
    receiver: Box<JsExpr>,
    dot_token: Token![.],
    method: Ident,
    paren_token: token::Paren,
    args: Punctuated<JsExpr, Token![,]>
}



pub fn parse_rsc(t: TokenStream) -> TokenStream {
    let t2 = t.clone();
    let before = std::time::Instant::now();
    let func: syn::File = syn::parse2(t2).map_err(|e| {
        println!("{:#?}", e)
    }).unwrap();
    times("file", before.elapsed());
    //let func = parse_macro_input!(t2 as Fn);
    //println!("{:#?}", func);
    t
}

pub fn tokens_from_file<P: AsRef<Path>>(path: P) -> TokenStream {
    use std::fs::File;
    use std::io::Read;
    use std::str::FromStr;
    let mut source_file = File::open(path).unwrap();
    let mut source = String::new();
    source_file.read_to_string(&mut source).unwrap();
    /*
    let mut b = String::new();
    for _ in 0..3000 {
        b.push_str(&source);
    }
    let before = std::time::Instant::now();*/
    let tokens = proc_macro2::TokenStream::from_str(&source).unwrap();
    //times("tokens", before.elapsed());
    //println!("{:#?}", tokens);
    tokens
}


fn main() {
    let t = tokens_from_file("scripts/minjs.rsc");
    println!("{:#?}", t);
}

fn times(msg: &str, d: std::time::Duration) {
    println!(
        "{}: {}",
        msg,
        (d.as_secs() as f32) + (d.as_nanos() as f32) / (1_000_000_000 as f32)
    );
}