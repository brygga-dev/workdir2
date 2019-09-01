use syn::{Result, parse::{Parse, ParseStream}};
use crate::item::Item;

pub struct File {
    items: Vec<Item>
}
impl Parse for File {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut items = Vec::new();
        while !input.is_empty() {
            items.push(input.parse()?);
        }
        Ok(File {
            items
        })
    }
}
