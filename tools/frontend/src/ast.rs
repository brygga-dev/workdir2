#[derive(Debug, PartialEq)]
pub enum Node {
    // Html tag
    Tag(Tag),
    Text(String),
    // (Name, Vec<(PropKey, Option<PropValue>))
    Component {
        name: String,
        props: Vec<(String, Option<String>)>,
        children: Vec<Node>,
    },
    // Inline script
    Script {
        script: String,
        typ: Option<String>,
    },
    // If this makes sense, maybe link stylesheet
    // also does
    ScriptSrc {
        src: String,
        defer: bool,
        asyc: bool,
        typ: Option<String>,
    },
    // Inline style
    Style(String),
    Comment(String),
}

#[derive(Debug, PartialEq)]
pub enum Tag {
    El {
        ident: El,
        attribs: Vec<Attrib>,
        children: Vec<Node>,
    },
    // Tag element with no children
    Void {
        ident: Void,
        attribs: Vec<Attrib>,
    },
    // Unrecognized tag
    Other {
        ident: String,
        attribs: Vec<Attrib>,
        children: Vec<Node>,
    },
}

// https://www.lifewire.com/html-singleton-tags-3468620
#[derive(Debug, PartialEq, Clone)]
pub enum Void {
    Img,
    Input,
    Br,
    Link,
    Meta,
    // Should maybe special case doctype
    Doctype,
    Source,
    Embed,
    Param,
    Command,
    Keygen,
    Hr,
    Area,
    Base,
    Col,
    Track,
    Wbr,
}
#[derive(Debug, PartialEq, Clone)]
pub enum El {
    Div,
    A,
    H1,
    H2,
    P,
    H3,
    H4,
    // These are top level
    //Script,
    //Style,
    Html,
    Head,
    Title,
    Body,
    Form,
    Select,
    Opt,
    Ul,
    Ol,
    Li,
    Table,
    // A few places only a subset of tags
    // makes sense
    // It would be helpful to encode this
    // Foremost table, ul, select
    Tr,
    Td,
    Th,
    Em,
    B,
    I,
    Header,
    Footer,
    Article,
    Aside,
    Main,
    Small,
    U,
    H5,
    H6,
    Nav,
}
pub fn void_str(el: &Void) -> &'static str {
    match el {
        Void::Img => "img",
        Void::Input => "input",
        Void::Br => "br",
        Void::Link => "link",
        Void::Meta => "meta",
        Void::Doctype => "doctype",
        Void::Source => "source",
        Void::Embed => "embed",
        Void::Param => "param",
        Void::Command => "command",
        Void::Keygen => "keygen",
        Void::Hr => "hr",
        Void::Area => "area",
        Void::Base => "base",
        Void::Col => "col",
        Void::Track => "track",
        Void::Wbr => "wbr",
    }
}
pub fn el_str(el: &El) -> &'static str {
    match el {
        El::Div => "div",
        El::A => "a",
        El::H1 => "h1",
        El::H2 => "h2",
        El::P => "p",
        El::H3 => "h3",
        El::H4 => "h4",
        El::Html => "html",
        El::Head => "head",
        El::Title => "title",
        El::Body => "body",
        El::Form => "form",
        El::Select => "select",
        El::Opt => "option",
        El::Ul => "ul",
        El::Ol => "ol",
        El::Li => "li",
        El::Table => "table",
        El::Tr => "tr",
        El::Td => "td",
        El::Th => "th",
        El::Em => "em",
        El::B => "b",
        El::I => "i",
        El::Header => "header",
        El::Footer => "footer",
        El::Article => "article",
        El::Aside => "aside",
        El::Main => "main",
        El::Small => "small",
        El::U => "u",
        El::H5 => "h5",
        El::H6 => "h6",
        El::Nav => "nav",
    }
}

/// Some attributes especially encoded,
/// common or meaningful
#[derive(Debug, PartialEq)]
pub enum Attrib {
    Id(String),
    Cls(String),
    OnClick(String),
    Href(String),
    Other(String, Option<String>),
}
