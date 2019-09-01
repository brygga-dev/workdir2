// There is definitely a lot of fickle and room
// for error in the code. It's driven by trying to
// create a really fast parser which helps in many areas.
// Html is maybe simple enough for this to be viable.
// There is possibly the question how it fares
// against some machine generated thing where this
// ordeal may not be worth it (or is after all)
// A nice thing about hand coding is I think it's
// easier to prioritize the likely cases

// Todos. Some way to better pinpoint errors. Could
// possibly use remaining length of slice to aid this.
// Maybe expand SliceExt more and do some tricks with
// generics and monomorphism to attach more information
// Extra info could be used to syntax highlight better.
// Also (very) interesting is context aware autocompletion.

use crate::ast::{el_str, void_str, Attrib, El, Node, Tag, Void};
use crate::parse_utils::*;

// The exclamation mark is assumed detected
fn parse_doctype(s: &[u8]) -> Result<(&[u8], Node)> {
    // todo, actual parsing
    let mut i = 0;
    while i < s.len() {
        if s.raw(i) == RBRACKET {
            return Ok((
                &s[i + 1..],
                Node::Tag(Tag::Void {
                    ident: Void::Doctype,
                    attribs: Vec::new(),
                }),
            ));
        }
        i += 1;
    }
    err("Failed to parse doctype", s)
}

// Stripping alot of whitespace,
// This is a decent approximation, but
// there may be cases for preserving the whitespace
// for example textareas (in that case apart from
// newline following start and preceding end tag)
// An empty space between elements may be
// significant also
// But we could maybe allow a stricter interpretation
// in this setting

pub fn parse_doc(s: &[u8]) -> Result<Vec<Node>> {
    let mut s = strip_space(s);
    let len = s.len();
    if len == 0 {
        // No nodes
        return Ok(Vec::new());
    }
    let mut nodes = Vec::new();
    // Branch to check for doctype
    // Tiny bit could be saved, but..
    // Would need connect with parse_children
    if let Some(s2) = s.parse_chr(LBRACKET) {
        // The first tag may be a doctype
        if len > 3 && s2.chr() == EXCLAMATION {
            let (s2, dt) = parse_doctype(&s2[1..])?;
            nodes.push(dt);
            s = s2;
        }
    }
    let (s, nodes) = parse_children(s, nodes)?;
    if s.len() > 0 {
        return err("Unexpected closing tag", s);
    }
    Ok(nodes)
}

fn parse_component(s: &[u8]) -> Result<(&[u8], Node)> {
    let (s2, ident) = push_ident_rest(&s[1..], sbuf_chr(s.chr() as char, 6));
    let mut s = s2;
    let mut props = Vec::new();
    // Props
    loop {
        s = strip_space(s);
        s.len_gt(0)?;
        let next = s.chr();
        s = &s[1..];
        match next {
            RBRACKET => {
                let (s2, children) = parse_children(s, Vec::new())?;
                if s2.len() < 1 || s2.chr() != LBRACKET {
                    return err("Expecting end tag", s2);
                }
                if let Some(s2) = &s2[1..].parse_chr_opt_space(FSLASH) {
                    if let Some(s2) = s2.parse_bstr_opt_space(ident.as_bytes()) {
                        return Ok((
                            s2,
                            Node::Component {
                                name: ident,
                                props,
                                children,
                            },
                        ));
                    }
                }
                return err("Expecting end tag", s2);
            }
            FSLASH => {
                if let Some(s2) = s.parse_chr_opt_space(RBRACKET) {
                    return Ok((
                        s2,
                        Node::Component {
                            name: ident,
                            props,
                            children: Vec::new(),
                        },
                    ));
                } else {
                    return err("Expecting '>'", s);
                }
            }
            other => {
                // Prop
                if is_alpha(other) {
                    return err("Expected alpha char as first in prop", s);
                }
                let (s2, ident) = push_ident_rest(s, sbuf_chr(other as char, 3));
                let (s2, attr_val) = parse_attr_value(s2)?;
                props.push((ident, attr_val));
                s = s2;
            }
        }
    }
}

/// Internal, "<script" has already been parsed
fn parse_script_tag(mut s: &[u8]) -> Result<(&[u8], Node)> {
    if let Some(s) = s.parse_chr_opt_space(RBRACKET) {
        // Script tag with script content
        let (s, script) = parse_quotable_content(s);
        if let Some(s) = s.parse_chr_opt_space(LBRACKET) {
            if let Some(s) = s.parse_chr_opt_space(FSLASH) {
                if let Some(s) = s.close_tag6(S, C, R, I, P, T) {
                    // Tag complete
                    return Ok((s, Node::Script { script, typ: None }));
                }
            }
        }
        return err("Expecting end tag", s);
    } else {
        // I wonder if we could use a macro for things
        // like these
        let mut src = None;
        let mut defer = false;
        let mut asyc = false;
        let mut typ = None;
        loop {
            s = strip_space(s);
            s.len_gt(0)?;
            let next = s.chr();
            s = &s[1..];
            match next {
                S => {
                    if let Some(s2) = s.parse_ident2([R, C]) {
                        let (s2, src_attr) = parse_attr_value(s2)?;
                        src = src_attr;
                        s = s2;
                    } else {
                        return err("Unrecognized script attribute", s);
                    }
                }
                D => {
                    if let Some(s2) = s.parse_ident4([E, F, E, R]) {
                        // Expecting none
                        let (s2, _attr) = parse_attr_value(s2)?;
                        defer = true;
                        s = s2;
                    } else {
                        return err("Unrecognized script attribute", s);
                    }
                }
                A => {
                    if let Some(s2) = s.parse_ident4([S, Y, N, C]) {
                        // Expecting none
                        let (s2, _attr) = parse_attr_value(s2)?;
                        asyc = true;
                        s = s2;
                    } else {
                        return err("Unrecognized script attribute", s);
                    }
                }
                T => {
                    if let Some(s2) = s.parse_ident3([Y, P, E]) {
                        let (s2, attr) = parse_attr_value(s2)?;
                        typ = attr;
                        s = s2;
                    } else {
                        return err("Unrecognized script attribute", s);
                    }
                }
                RBRACKET => {
                    if let Some(s) = s.parse_chr_opt_space(LBRACKET) {
                        // <script src=..> type tag
                        if let Some(s) = s.parse_chr_opt_space(FSLASH) {
                            if let Some(s) = s.close_tag6(S, C, R, I, P, T) {
                                match src {
                                    Some(src) => {
                                        // Tag complete
                                        return Ok((
                                            s,
                                            Node::ScriptSrc {
                                                src,
                                                defer,
                                                asyc,
                                                typ,
                                            },
                                        ));
                                    }
                                    None => return err("Src attribute expected", s),
                                }
                            }
                        }
                    } else {
                        // This could be <script type=..>..</script>
                        s = strip_space(s);
                        let (s, script) = parse_quotable_content(s);
                        if let Some(s) = s.parse_chr_opt_space(LBRACKET) {
                            if let Some(s) = s.parse_chr_opt_space(FSLASH) {
                                if let Some(s) = s.close_tag6(S, C, R, I, P, T) {
                                    // Tag complete
                                    return Ok((s, Node::Script { script, typ }));
                                }
                            }
                        }
                    }
                    return err("Expecting end tag", s);
                }
                // This may not be supported in html
                FSLASH => {
                    if let Some(s) = s.parse_chr_opt_space(RBRACKET) {
                        // Tag complete
                        match src {
                            Some(src) => {
                                return Ok((
                                    s,
                                    Node::ScriptSrc {
                                        src,
                                        defer,
                                        asyc,
                                        typ,
                                    },
                                ));
                            }
                            None => return err("Script src required", s),
                        }
                    }
                    return err("Expecting closing tag", s);
                }
                _ => return err("Unrecognized script attribute", s),
            }
        }
    }
}

/// Internal, "<style" has already been parsed
fn parse_style_tag(mut s: &[u8]) -> Result<(&[u8], Node)> {
    if let Some(s) = s.parse_chr_opt_space(RBRACKET) {
        let s = strip_space(s);
        let (s, style) = parse_quotable_content(s);
        // End tag
        if let Some(s) = s.parse_chr_opt_space(LBRACKET) {
            if let Some(s) = s.parse_chr_opt_space(FSLASH) {
                if let Some(s) = s.close_tag5(S, T, Y, L, E) {
                    // Tag complete
                    return Ok((s, Node::Style(style)));
                }
            }
        }
        return err("Expecting style end tag", s);
    } else {
        // Currently to not fail on type attribute
        let mut _typ = None;
        loop {
            s = strip_space(s);
            s.len_gt(0)?;
            let next = s.chr();
            s = &s[1..];
            match next {
                T => {
                    if let Some(s2) = s.parse_ident3([Y, P, E]) {
                        let (s2, attr) = parse_attr_value(s2)?;
                        _typ = attr;
                        s = s2;
                    } else {
                        return err("Unrecognized style attribute", s);
                    }
                }
                RBRACKET => {
                    let s = strip_space(s);
                    let (s, style) = parse_quotable_content(s);
                    if let Some(s) = s.parse_chr_opt_space(LBRACKET) {
                        // <style type=..> type tag
                        // End tag
                        if let Some(s) = s.parse_chr_opt_space(LBRACKET) {
                            if let Some(s) = s.parse_chr_opt_space(FSLASH) {
                                if let Some(s) = s.close_tag5(S, T, Y, L, E) {
                                    // Tag complete
                                    return Ok((s, Node::Style(style)));
                                }
                            }
                        }
                        return err("Expecting style end tag", s);
                    }
                    return err("Expecting end tag", s);
                }
                FSLASH => return err("Unexpected / in style tag", s),
                _ => return err("Unrecognized attribute in style tag", s),
            }
        }
    }
}

/// Parses a list of children until closing tag or
/// end of bytes
/// Detects node type
pub fn parse_children(mut s: &[u8], mut nodes: Vec<Node>) -> Result<(&[u8], Vec<Node>)> {
    // Loop and collect elements
    loop {
        s = strip_space(s);
        let len = s.len();
        if len == 0 {
            return Ok((s, nodes));
        }
        // If '<' and tag possible (min 3 chars)
        if s.chr() == LBRACKET && len >= 3 {
            s = &s[1..];
            // There may be spaces here, though the likely
            // case is ident or /
            if is_space(s.chr()) {
                s = strip_space(s);
                s.len_gt(0)?;
            }
            if s.is_uc() {
                // Expecting Component
                // todo: Consider lib.my.comps.Comp notation
                let (s2, component) = parse_component(s)?;
                nodes.push(component);
                s = s2;
                continue;
            }
            match s.chr() {
                FSLASH => {
                    // This could be explicitly closing a children assumed as void?
                    // Or closing further up the chain
                    // Could be relevant to <p> tag which some use as space
                    // Currently passing back control to parse_element
                    return Ok((&s[1..], nodes));
                }
                S => {
                    // Special case script/style
                    if let Some(s2) = &s[1..].parse_ident5([C, R, I, P, T]) {
                        let (s2, script) = parse_script_tag(s2)?;
                        nodes.push(script);
                        s = s2;
                        continue;
                    } else if let Some(s2) = &s[1..].parse_ident4([T, Y, L, E]) {
                        let (s2, style) = parse_style_tag(s2)?;
                        nodes.push(style);
                        s = s2;
                        continue;
                    } else {
                        let (s2, tag) = parse_tag(s)?;
                        nodes.push(tag);
                        s = s2;
                        continue;
                    }
                }
                EXCLAMATION => {
                    if let Some(s2) = &s[1..].parse_bstr(b"--") {
                        // Comment
                        // In some instances I guess this could/should be stripped
                        // Parse until -->
                        let mut b = String::new();
                        let mut consumed = 0;
                        while let Some((size, chr)) = next_utf8(&s2[consumed..]) {
                            consumed += size;
                            if chr == '>'
                                && s2.raw(consumed - 2) == DASH
                                && s2.raw(consumed - 3) == DASH
                            {
                                // Found end, truncate last two dashes
                                b.truncate(b.len() - 2);
                                break;
                            }
                            b.push(chr);
                        }
                        s = &s2[consumed..];
                        nodes.push(Node::Comment(b));
                    } else {
                        return err("Unrecognized <!", s);
                    }
                }
                _ => {
                    let (s2, tag) = parse_tag(s)?;
                    nodes.push(tag);
                    s = s2;
                    continue;
                }
            }
        } else {
            // Text node
            let (s2, text_node) = parse_text(s);
            nodes.push(text_node);
            s = s2;
        }
    }
}

// Expecting an opened element, and tag type detected
// If it is a Void, or self-closed element it will
// return a single element, for regular elements it will
// collect children
pub fn parse_tag(s: &[u8]) -> Result<(&[u8], Node)> {
    let (s, ident) = parse_tag_ident(s);
    match ident {
        ParsedTag::El(ident) => {
            let (s, self_closed, attribs) = parse_attributes(s)?;
            if self_closed {
                Ok((
                    s,
                    Node::Tag(Tag::El {
                        ident,
                        attribs,
                        children: Vec::new(),
                    }),
                ))
            } else {
                let (s, children) = parse_children(s, Vec::with_capacity(2))?;
                if s.len() == 0 {
                    return err(
                        format!("Eof, expecting close tag for: {}", el_str(&ident)),
                        s,
                    );
                }
                let close_tag = match ident {
                    El::Div => s.close_tag3(D, I, V),
                    El::A => s.close_tag1(A),
                    El::H1 => s.close_tag2(H, N1),
                    El::H2 => s.close_tag2(H, N2),
                    El::P => s.close_tag1(P),
                    El::H3 => s.close_tag2(H, N3),
                    El::H4 => s.close_tag2(H, N4),
                    El::Html => s.close_tag4(H, T, M, L),
                    El::Head => s.close_tag4(H, E, A, D),
                    El::Title => s.close_tag5(T, I, T, L, E),
                    El::Body => s.close_tag4(B, O, D, Y),
                    El::Form => s.close_tag4(F, O, R, M),
                    El::Select => s.close_tag6(S, E, L, E, C, T),
                    El::Opt => s.close_tag6(O, P, T, I, O, N),
                    El::Ul => s.close_tag2(U, L),
                    El::Ol => s.close_tag2(O, L),
                    El::Li => s.close_tag2(L, I),
                    El::Table => s.close_tag5(T, A, B, L, E),
                    El::Tr => s.close_tag2(T, R),
                    El::Td => s.close_tag2(T, D),
                    El::Th => s.close_tag2(T, H),
                    El::Em => s.close_tag2(E, M),
                    El::B => s.close_tag1(B),
                    El::I => s.close_tag1(I),
                    El::Header => s.close_tag6(H, E, A, D, E, R),
                    El::Footer => s.close_tag6(F, O, O, T, E, R),
                    El::Article => s.close_tag7(A, R, T, I, C, L, E),
                    El::Aside => s.close_tag5(A, S, I, D, E),
                    El::Main => s.close_tag4(M, A, I, N),
                    El::Small => s.close_tag5(S, M, A, L, L),
                    El::U => s.close_tag1(U),
                    El::H5 => s.close_tag2(H, N5),
                    El::H6 => s.close_tag2(H, N6),
                    El::Nav => s.close_tag3(N, A, V),
                };
                match close_tag {
                    Some(s) => Ok((
                        s,
                        Node::Tag(Tag::El {
                            ident,
                            attribs,
                            children,
                        }),
                    )),
                    None => err(format!("Expecting close tag for: {}", el_str(&ident)), s),
                }
            }
        }
        ParsedTag::Void(ident) => {
            let (s, _, attribs) = parse_attributes(s)?;
            Ok((s, Node::Tag(Tag::Void { ident, attribs })))
        }
        ParsedTag::Other(ident) => {
            let (s, self_closed, attribs) = parse_attributes(s)?;
            if self_closed {
                Ok((
                    s,
                    Node::Tag(Tag::Other {
                        ident,
                        attribs,
                        children: Vec::new(),
                    }),
                ))
            } else {
                let (s, children) = parse_children(s, Vec::with_capacity(2))?;
                if s.len() == 0 {
                    return err("Expected closing tag", s);
                }
                if let Some(s) = s.parse_bstr_opt_space(ident.as_bytes()) {
                    match s.parse_chr_opt_space(RBRACKET) {
                        Some(s) => Ok((
                            s,
                            Node::Tag(Tag::Other {
                                ident,
                                attribs,
                                children,
                            }),
                        )),
                        None => err("Closing tag not complete", s),
                    }
                } else {
                    err(format!("Expected closing tag: {}", ident), s)
                }
            }
        }
    }
}

enum ParsedTag {
    El(El),
    Void(Void),
    Other(String),
}
/// Optimized tag ident parser that detects
/// void elements and components by capital
/// first letter as it works in jsx
fn parse_tag_ident(s: &[u8]) -> (&[u8], ParsedTag) {
    // Assuming space stripped and len_gt(0) as current
    // char should've been used to determine type
    // Some optimized parsing for some reason
    let first_chr = s.chr();
    // This is for after first char
    let s = &s[1..];
    let len = s.len();
    match first_chr {
        // Div
        D => {
            if let Some(s) = s.parse_ident2([I, V]) {
                (s, ParsedTag::El(El::Div))
            } else {
                el_ident_rest(s, sbuf_chr('d', 6))
            }
        }
        // Img, input
        I => {
            if let Some(s) = s.parse_ident2([M, G]) {
                (s, ParsedTag::Void(Void::Img))
            } else if let Some(s) = s.parse_ident4([N, P, U, T]) {
                (s, ParsedTag::Void(Void::Input))
            } else if len > 0 && s.is_ident() {
                // Fallback
                el_ident_rest(s, sbuf_chr('i', 6))
            } else {
                // Plain I element
                (s, ParsedTag::El(El::I))
            }
        }
        // A, Area, Article, Aside
        A => {
            if len > 0 && s.is_ident() {
                if let Some(s) = s.parse_ident3([R, E, A]) {
                    (s, ParsedTag::Void(Void::Area))
                } else if let Some(s) = s.parse_ident6([R, T, I, C, L, E]) {
                    (s, ParsedTag::El(El::Article))
                } else if let Some(s) = s.parse_ident4([S, I, D, E]) {
                    (s, ParsedTag::El(El::Aside))
                } else {
                    el_ident_rest(s, sbuf_chr('a', 12))
                }
            } else {
                // A tag
                (s, ParsedTag::El(El::A))
            }
        }
        // Br, B, Body, Base
        B => {
            if len > 0 && s.is_ident() {
                if let Some(s) = s.parse_ident1(R) {
                    (s, ParsedTag::Void(Void::Br))
                } else if let Some(s) = s.parse_ident3([O, D, Y]) {
                    (s, ParsedTag::El(El::Body))
                } else if let Some(s) = s.parse_ident3([A, S, E]) {
                    (s, ParsedTag::Void(Void::Base))
                } else {
                    el_ident_rest(s, sbuf_chr('b', 7))
                }
            } else {
                // B tag
                (s, ParsedTag::El(El::B))
            }
        }
        // H1-H6, HR, HTML, HEAD, Header
        H => {
            // Check for h1-h6 and hr
            if len > 1 && !is_ident_char(s.raw(1)) {
                // Second char is non-ident
                match s.chr() {
                    N1 => (&s[1..], ParsedTag::El(El::H1)),
                    N2 => (&s[1..], ParsedTag::El(El::H2)),
                    N3 => (&s[1..], ParsedTag::El(El::H3)),
                    N4 => (&s[1..], ParsedTag::El(El::H4)),
                    R => (&s[1..], ParsedTag::Void(Void::Hr)),
                    N5 => (&s[1..], ParsedTag::El(El::H5)),
                    N6 => (&s[1..], ParsedTag::El(El::H6)),
                    other => {
                        // Some other ident, h + other
                        let mut b = sbuf_chr('h', 2);
                        b.push(other as char);
                        (&s[1..], ParsedTag::Other(b))
                    }
                }
            } else if let Some(s) = s.parse_ident3([T, M, L]) {
                (s, ParsedTag::El(El::Html))
            } else if let Some(s) = s.parse_ident3([E, A, D]) {
                (s, ParsedTag::El(El::Head))
            } else if let Some(s) = s.parse_ident5([E, A, D, E, R]) {
                (s, ParsedTag::El(El::Header))
            } else {
                // Fallback H..
                el_ident_rest(s, sbuf_chr('h', 6))
            }
        }
        // (Script, Style), Select, Small, Source
        S => {
            if let Some(s) = s.parse_ident5([E, L, E, C, T]) {
                (s, ParsedTag::El(El::Select))
            } else if let Some(s) = s.parse_ident4([M, A, L, L]) {
                (s, ParsedTag::El(El::Small))
            } else if let Some(s) = s.parse_ident5([O, U, R, C, E]) {
                (s, ParsedTag::Void(Void::Source))
            } else {
                el_ident_rest(s, sbuf_chr('s', 6))
            }
        }
        // Td, Tr, Th, Table, Title, Track
        T => {
            if let Some(s) = s.parse_ident1(D) {
                (s, ParsedTag::El(El::Td))
            } else if let Some(s) = s.parse_ident1(R) {
                (s, ParsedTag::El(El::Tr))
            } else if let Some(s) = s.parse_ident1(H) {
                (s, ParsedTag::El(El::Th))
            } else if let Some(s) = s.parse_ident4([A, B, L, E]) {
                (s, ParsedTag::El(El::Table))
            } else if let Some(s) = s.parse_ident4([I, T, L, E]) {
                (s, ParsedTag::El(El::Title))
            } else if let Some(s) = s.parse_ident4([R, A, C, K]) {
                (s, ParsedTag::Void(Void::Track))
            } else {
                el_ident_rest(s, sbuf_chr('t', 6))
            }
        }
        // Option, Ol
        O => {
            if let Some(s) = s.parse_ident5([P, T, I, O, N]) {
                (s, ParsedTag::El(El::Opt))
            } else if let Some(s) = s.parse_ident1(L) {
                (s, ParsedTag::El(El::Ol))
            } else {
                el_ident_rest(s, sbuf_chr('o', 6))
            }
        }
        // P, Param
        P => {
            if len > 0 && s.is_ident() {
                if let Some(s) = s.parse_ident4([A, R, A, M]) {
                    (s, ParsedTag::Void(Void::Param))
                } else {
                    el_ident_rest(s, sbuf_chr('p', 6))
                }
            } else {
                // Plain P
                (s, ParsedTag::El(El::P))
            }
        }
        // Li, Link
        L => {
            if let Some(s) = s.parse_ident1(I) {
                (s, ParsedTag::El(El::Li))
            } else if let Some(s) = s.parse_ident3([I, N, K]) {
                (s, ParsedTag::Void(Void::Link))
            } else {
                el_ident_rest(s, sbuf_chr('l', 6))
            }
        }
        // Form, Footer
        F => {
            if let Some(s) = s.parse_ident3([O, R, M]) {
                (s, ParsedTag::El(El::Form))
            } else if let Some(s) = s.parse_ident5([O, O, T, E, R]) {
                (s, ParsedTag::El(El::Footer))
            } else {
                el_ident_rest(s, sbuf_chr('f', 6))
            }
        }
        // Meta, Main
        M => {
            if let Some(s) = s.parse_ident3([E, T, A]) {
                (s, ParsedTag::Void(Void::Meta))
            } else if let Some(s) = s.parse_ident3([A, I, N]) {
                (s, ParsedTag::El(El::Main))
            } else {
                el_ident_rest(s, sbuf_chr('m', 6))
            }
        }
        // Ul, U
        U => {
            if let Some(s) = s.parse_ident1(L) {
                (s, ParsedTag::El(El::Ul))
            } else if len > 0 && s.is_ident() {
                el_ident_rest(s, sbuf_chr('u', 6))
            } else {
                // U element
                (s, ParsedTag::El(El::U))
            }
        }
        // Nav
        N => {
            if let Some(s) = s.parse_ident2([A, V]) {
                (s, ParsedTag::El(El::Nav))
            } else {
                el_ident_rest(s, sbuf_chr('n', 6))
            }
        }
        // Embed, Em
        E => {
            if let Some(s) = s.parse_ident1(M) {
                (s, ParsedTag::El(El::Em))
            } else if let Some(s) = s.parse_ident4([M, B, E, D]) {
                (s, ParsedTag::Void(Void::Embed))
            } else {
                el_ident_rest(s, sbuf_chr('e', 6))
            }
        }
        // Command, Col
        C => {
            if let Some(s) = s.parse_ident6([O, M, M, A, N, D]) {
                (s, ParsedTag::Void(Void::Command))
            } else if let Some(s) = s.parse_ident2([O, L]) {
                (s, ParsedTag::Void(Void::Col))
            } else {
                el_ident_rest(s, sbuf_chr('c', 6))
            }
        }
        // Keygen
        K => {
            if let Some(s) = s.parse_ident5([E, Y, G, E, N]) {
                (s, ParsedTag::Void(Void::Keygen))
            } else {
                el_ident_rest(s, sbuf_chr('k', 6))
            }
        }
        // Wbr
        W => {
            if let Some(s) = s.parse_ident2([B, R]) {
                (s, ParsedTag::Void(Void::Wbr))
            } else {
                el_ident_rest(s, sbuf_chr('w', 6))
            }
        }
        // No match for first character
        other => el_ident_rest(s, sbuf_chr(other as char, 6)),
    }
}

/// Parses attribs until end of tag
/// This will detect '>', so passes on
/// Returns (self_closed, vec<attr, attr_value>)
pub fn parse_attributes(mut s: &[u8]) -> Result<(&[u8], bool, Vec<Attrib>)> {
    let mut attribs = Vec::new();
    loop {
        s = strip_space(s);
        s.len_gt(0)?;
        let next = s.chr();
        s = &s[1..];
        match next {
            RBRACKET => return Ok((s, false, attribs)), // Done
            FSLASH => {
                if let Some(s) = s.parse_chr_opt_space(RBRACKET) {
                    return Ok((s, true, attribs));
                } else {
                    return err("Expected '>' after '/'", s);
                }
            }
            I => {
                s = if let Some(s2) = s.parse_ident1(D) {
                    let (s2, attr_val) = parse_attr_value(s2)?;
                    match attr_val {
                        Some(id) => attribs.push(Attrib::Id(id)),
                        None => return err("Id attribute requires value", s),
                    }
                    s2
                } else {
                    let (s2, ident) = push_ident_rest(s, sbuf_chr('i', 4));
                    let (s2, attr_val) = parse_attr_value(s2)?;
                    attribs.push(Attrib::Other(ident, attr_val));
                    s2
                };
            }
            C => {
                s = if let Some(s2) = s.parse_ident4([L, A, S, S]) {
                    // Could use specialized parser here
                    // to get list of classes
                    let (s2, attr_val) = parse_attr_value(s2)?;
                    match attr_val {
                        Some(class_list) => attribs.push(Attrib::Cls(class_list)),
                        None => return err("Class attribute requires value", s),
                    }
                    s2
                } else {
                    let (s2, ident) = push_ident_rest(s, sbuf_chr('c', 4));
                    let (s2, attr_val) = parse_attr_value(s2)?;
                    attribs.push(Attrib::Other(ident, attr_val));
                    s2
                };
            }
            O => {
                s = if let Some(s2) = s.parse_ident6([N, C, L, I, C, K]) {
                    let (s2, attr_val) = parse_attr_value(s2)?;
                    match attr_val {
                        Some(on_click) => attribs.push(Attrib::OnClick(on_click)),
                        None => return err("Onclick attribute requires value", s),
                    }
                    s2
                } else {
                    let (s2, ident) = push_ident_rest(s, sbuf_chr('o', 5));
                    let (s2, attr_val) = parse_attr_value(s2)?;
                    attribs.push(Attrib::Other(ident, attr_val));
                    s2
                };
            }
            H => {
                s = if let Some(s2) = s.parse_ident3([R, E, F]) {
                    let (s2, attr_val) = parse_attr_value(s2)?;
                    match attr_val {
                        Some(href) => attribs.push(Attrib::Href(href)),
                        None => return err("Href attribute requires value", s),
                    }
                    s2
                } else {
                    let (s2, ident) = push_ident_rest(s, sbuf_chr('h', 5));
                    let (s2, attr_val) = parse_attr_value(s2)?;
                    attribs.push(Attrib::Other(ident, attr_val));
                    s2
                };
            }
            other => {
                let (s2, ident) = push_ident_rest(s, sbuf_chr(other as char, 5));
                let (s2, attr_val) = parse_attr_value(s2)?;
                attribs.push(Attrib::Other(ident, attr_val));
                s = s2;
            }
        }
    }
}

/// Parse potential attrib value from ="val"
/// Will not strip trailing space
pub fn parse_attr_value(s: &[u8]) -> Result<(&[u8], Option<String>)> {
    let len = s.len();
    if len == 0 {
        return Ok((s, None));
    }
    // Some attribs does not require a value
    if let Some(mut s) = s.parse_chr_opt_space(EQUAL) {
        s.len_gt(0)?;
        let mut b = String::with_capacity(3);
        loop {
            match s.chr() {
                DQUOTE => {
                    // Escaping in attributes is by using &quot;
                    // We need to iterate chars to account
                    // for unicodes that could end in equalent
                    // of DQUOTE.
                    let mut consumed = 1;
                    // Todo: Not sure if these add's are more efficient
                    while let Some((size, chr)) = next_utf8(&s[consumed..]) {
                        consumed += size;
                        if chr == '"' {
                            return Ok((&s[consumed..], Some(b)));
                        } else {
                            b.push(chr);
                        }
                    }
                    // Didn't find ending quote,
                    return err("Expected ending double quote", s);
                }
                SQUOTE => {
                    let mut consumed = 1;
                    while let Some((size, chr)) = next_utf8(&s[consumed..]) {
                        consumed += size;
                        if chr == '\'' {
                            return Ok((&s[consumed..], Some(b)));
                        } else {
                            b.push(chr);
                        }
                    }
                    // Didn't find ending quote,
                    return err("Expected ending single quote", s);
                }
                SPACE | NL | TAB | CR => {
                    s = strip_space(&s[1..]);
                    if s.len() == 0 {
                        // Equal char with no attrib value..
                        // interpreting as empty
                        return Ok((s, Some(b)));
                    }
                    // Should get match on next iteration
                }
                _ => {
                    // Unquoted attributes, consume until whitespace
                    // or > or /
                    let mut consumed = 0;
                    while let Some((size, chr)) = next_utf8(&s[consumed..]) {
                        consumed += size;
                        match chr {
                            ' ' | '>' | '/' | '\t' | '\n' | '\r' => {
                                return Ok((&s[consumed..], Some(b)))
                            }
                            other => b.push(other),
                        }
                    }
                    return err("Expected end of unquoted attribute", s);
                }
            }
        }
    } else {
        // No equal sign, "value less" attrib
        return Ok((s, None));
    }
}

/// Parses a text node
pub fn parse_text(mut s: &[u8]) -> (&[u8], Node) {
    let mut b = String::with_capacity(32);
    while let Some((size, chr)) = next_utf8(s) {
        // Break on '<'
        if chr == '<' {
            break;
        }
        b.push(chr);
        s = &s[size..];
    }
    // I think it makes sense to trim backwards here
    // for space efficiency and consistency with
    // trimmed left
    let mut blank_size = 0;
    for c in b.chars().rev() {
        if !c.is_ascii() {
            break;
        }
        match c as u8 {
            SPACE => blank_size += 1,
            TAB => blank_size += 1,
            NL => blank_size += 1,
            CR => blank_size += 1,
            _ => break,
        }
    }
    if blank_size > 0 {
        b.truncate(b.len() - blank_size);
    }
    (s, Node::Text(b))
}

/// Captures script/style content to a string
/// Will capture until encountering a '<', unless
/// this is within a quoted string
pub fn parse_quotable_content(s: &[u8]) -> (&[u8], String) {
    let mut b = String::with_capacity(32);
    let mut i = 0;
    while let Some((size, chr)) = next_utf8(&s[i..]) {
        match chr {
            '<' => {
                // This may also be less than operator,
                // check for end tag
                // I think this is sufficient, (some weird
                // end of line comment might happen)
                if let Some(s2) = &s[i + size..].parse_chr_opt_space(FSLASH) {
                    if s2.len() > 0 && s2.chr() != FSLASH {
                        break;
                    }
                }
                // Above check didn't find end tag
                i += size;
                b.push('<');
            }
            // Quoted sections
            '"' => {
                i += size;
                b.push('"');
                // Loop until unescaped quote
                while let Some((size, chr)) = next_utf8(&s[i..]) {
                    b.push(chr);
                    i += size;
                    match chr {
                        '"' => break,
                        '\\' => {
                            // Consume next
                            if let Some((size, chr)) = next_utf8(&s[i..]) {
                                i += size;
                                b.push(chr);
                            }
                        }
                        _ => (),
                    }
                }
            }
            '\'' => {
                i += size;
                b.push('\'');
                // Loop until unescaped quote
                while let Some((size, chr)) = next_utf8(&s[i..]) {
                    b.push(chr);
                    i += size;
                    match chr {
                        '\'' => break,
                        '\\' => {
                            // Consume next
                            if let Some((size, chr)) = next_utf8(&s[i..]) {
                                i += size;
                                b.push(chr);
                            }
                        }
                        _ => (),
                    }
                }
            }
            '`' => {
                i += size;
                b.push('`');
                // Loop until unescaped quote
                while let Some((size, chr)) = next_utf8(&s[i..]) {
                    b.push(chr);
                    i += size;
                    match chr {
                        '`' => break,
                        '\\' => {
                            // Consume next
                            if let Some((size, chr)) = next_utf8(&s[i..]) {
                                i += size;
                                b.push(chr);
                            }
                        }
                        _ => (),
                    }
                }
            }
            _ => {
                b.push(chr);
                i += size;
            }
        }
    }
    (&s[i..], b)
}

// not used
/// Identifier as starting with alpha character,
/// then any alpha, numeric or dash
/// Intended for tag and attribute names
pub fn parse_ident(s: &[u8]) -> Result<(&[u8], String)> {
    s.len_gt(0)?;
    if is_alpha(s.chr()) {
        return err("Expected alpha char as first in identifier", s);
    }
    Ok(push_ident_rest(&s[1..], sbuf_chr(s.chr() as char, 3)))
}

/// Helper to add rest of ident valid characters
#[inline]
fn push_ident_rest(s: &[u8], mut b: String) -> (&[u8], String) {
    let len = s.len();
    let mut i = 0;
    while i < len {
        let chr = s.raw(i);
        if !is_ident_char(chr) {
            break;
        }
        b.push(chr as char);
        i += 1;
    }
    (&s[i..], b)
}

// Aiding ident parser
#[inline]
fn el_ident_rest(s: &[u8], b: String) -> (&[u8], ParsedTag) {
    let (s, ident) = push_ident_rest(s, b);
    return (s, ParsedTag::Other(ident));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_empty() {
        let res = parse_doc(b"");
        assert_eq!(Ok(Vec::new()), res);
    }

    #[test]
    fn test_doctype() {
        let res = parse_doc(b"<!DOCTYPE html>");
        assert_eq!(
            Ok(vec![Node::Tag(Tag::Void {
                ident: Void::Doctype,
                attribs: Vec::new()
            })]),
            res
        );
        // Doctype then tag
        let res = parse_doc(b"<!DOCTYPE html><div></div>");
        assert_eq!(
            Ok(vec![
                Node::Tag(Tag::Void {
                    ident: Void::Doctype,
                    attribs: Vec::new()
                }),
                Node::Tag(Tag::El {
                    ident: El::Div,
                    attribs: Vec::new(),
                    children: Vec::new()
                })
            ]),
            res
        );
    }

    #[test]
    fn test_simple_tag() {
        let res = parse_doc(b"<div></div>");
        assert_eq!(
            Ok(vec![Node::Tag(Tag::El {
                ident: El::Div,
                attribs: Vec::new(),
                children: Vec::new()
            })]),
            res
        );
    }

    #[test]
    fn test_void_tags() {
        let tags = void_tags();
        for tag in tags {
            let tag_str = crate::ast::void_str(&tag);
            let doc = format!("<{}>", tag_str);
            let res = parse_doc(doc.as_bytes());
            assert_eq!(
                Ok(vec![Node::Tag(Tag::Void {
                    ident: tag,
                    attribs: Vec::new()
                })]),
                res
            );
        }
    }

    #[test]
    fn test_el_tags() {
        let tags = el_tags();
        for tag in tags {
            let tag_str = crate::ast::el_str(&tag);
            for doc in &[
                format!("<{}/>", tag_str),
                format!("<{}></{}>", tag_str, tag_str),
            ] {
                let res = parse_doc(doc.as_bytes());
                assert_eq!(
                    Ok(vec![Node::Tag(Tag::El {
                        ident: tag.clone(),
                        attribs: Vec::new(),
                        children: Vec::new()
                    })]),
                    res
                );
            }
        }
    }

    #[test]
    fn test_attrib() {
        // Single attrib
        let tags = void_tags();
        for tag in tags {
            let tag_str = crate::ast::void_str(&tag);
            let doc = format!("<{} custom=\"value\">", tag_str);
            let res = parse_doc(doc.as_bytes());
            assert_eq!(
                Ok(vec![Node::Tag(Tag::Void {
                    ident: tag,
                    attribs: vec![Attrib::Other("custom".into(), Some("value".into()))]
                })]),
                res
            );
        }
        let tags = el_tags();
        for tag in tags {
            let tag_str = crate::ast::el_str(&tag);
            let doc = format!("<{} custom=\"value\"/>", tag_str);
            let res = parse_doc(doc.as_bytes());
            assert_eq!(
                Ok(vec![Node::Tag(Tag::El {
                    ident: tag,
                    attribs: vec![Attrib::Other("custom".into(), Some("value".into()))],
                    children: Vec::new()
                })]),
                res
            );
        }
    }

    #[test]
    fn test_attribs() {
        // Multiple attrib
        let tags = void_tags();
        for tag in tags {
            let tag_str = crate::ast::void_str(&tag);
            let doc = format!(
                r#"<{}
                    id="id-val"
                    class="cls-val cls-val2"
                    onclick ="onclick-val"
                    href= "https://example.com"
                    custom="value"
                    singlequote='sq'
                    noquote= ok noquote2=ok2 >"#,
                tag_str
            );
            let res = parse_doc(doc.as_bytes());
            assert_eq!(
                Ok(vec![Node::Tag(Tag::Void {
                    ident: tag,
                    attribs: vec![
                        Attrib::Id("id-val".into()),
                        Attrib::Cls("cls-val cls-val2".into()),
                        Attrib::OnClick("onclick-val".into()),
                        Attrib::Href("https://example.com".into()),
                        Attrib::Other("custom".into(), Some("value".into())),
                        Attrib::Other("singlequote".into(), Some("sq".into())),
                        Attrib::Other("noquote".into(), Some("ok".into())),
                        Attrib::Other("noquote2".into(), Some("ok2".into())),
                    ]
                })]),
                res
            );
        }
    }

    #[test]
    fn test_children() {
        let doc = r#"<div><hr></div>"#;
        let res = parse_doc(doc.as_bytes());
        assert_eq!(
            Ok(vec![Node::Tag(Tag::El {
                ident: El::Div,
                attribs: Vec::new(),
                children: vec![Node::Tag(Tag::Void {
                    ident: Void::Hr,
                    attribs: Vec::new()
                })]
            })]),
            res
        );
        let doc = r#"<div><hr><hr></div>"#;
        let res = parse_doc(doc.as_bytes());
        assert_eq!(
            Ok(vec![Node::Tag(Tag::El {
                ident: El::Div,
                attribs: Vec::new(),
                children: vec![
                    Node::Tag(Tag::Void {
                        ident: Void::Hr,
                        attribs: Vec::new()
                    }),
                    Node::Tag(Tag::Void {
                        ident: Void::Hr,
                        attribs: Vec::new()
                    })
                ]
            })]),
            res
        );
        let doc = r#"<ul><li>Item1</li><li>Item2</li></ul>"#;
        let res = parse_doc(doc.as_bytes());
        assert_eq!(
            Ok(vec![Node::Tag(Tag::El {
                ident: El::Ul,
                attribs: Vec::new(),
                children: vec![
                    Node::Tag(Tag::El {
                        ident: El::Li,
                        attribs: Vec::new(),
                        children: vec![Node::Text("Item1".into())]
                    }),
                    Node::Tag(Tag::El {
                        ident: El::Li,
                        attribs: Vec::new(),
                        children: vec![Node::Text("Item2".into())]
                    }),
                ]
            })]),
            res
        );
    }

    #[test]
    fn test_script() {
        let script = r#"
            let x = 1;
            let y = "\"";
            let z = "Ok</script>";
        "#;
        let res = parse_doc(
            format!(
                "<script>{}</script>",
                script
            )
            .as_bytes(),
        );
        assert_eq!(
            Ok(vec![Node::Script {
                script: script.into(),
                typ: None
            }]),
            res
        );
    }

    #[test]
    fn test_style() {
        let style = r#"#an-id {font-weight: bold;}"#;
        let res = parse_doc(
            format!(
                "<style>{}</style>",
                style
            )
            .as_bytes(),
        );
        assert_eq!(Ok(vec![Node::Style(style.into())]), res);
    }

    #[test]
    fn test_stylesheet_link() {
        let doc = r#"<link
            rel='stylesheet'
            id='front-css'
            href='https://example.com/wp-content/themes/theme/css/front.css'
            type='text/css'
            media='all' />"#;
        let res = parse_doc(doc.as_bytes());
        assert_eq!(
            Ok(vec![Node::Tag(Tag::Void {
                ident: Void::Link,
                attribs: vec![
                    Attrib::Other("rel".into(), Some("stylesheet".into())),
                    Attrib::Id("front-css".into()),
                    Attrib::Href(
                        "https://example.com/wp-content/themes/theme/css/front.css".into()
                    ),
                    Attrib::Other("type".into(), Some("text/css".into())),
                    Attrib::Other("media".into(), Some("all".into())),
                ]
            })]),
            res
        );
    }

    fn void_tags() -> Vec<Void> {
        use Void::*;
        vec![
            Img, Input, Br, Link, Meta, Source, Embed, Param, Command, Keygen, Hr, Area, Base, Col,
            Track, Wbr,
        ]
    }

    fn el_tags() -> Vec<El> {
        use El::*;
        vec![
            Div, A, H1, H2, P, H3, H4, Html, Head, Title, Body, Form, Select, Opt, Ul, Ol, Li,
            Table, Tr, Td, Th, Em, B, I, Header, Footer, Article, Aside, Main, Small, U, H5, H6,
            Nav,
        ]
    }
}
