use crate::php::{PhpValue, ArrKey};

// Meant (at least) for git stored data
// Increases single line diffs, making changes
// easier to read, and reduces diff size
// Compared to json, we encode multiline strings
// and allow utf-8 here
pub fn serialize_php(mut b: String, value: PhpValue, indent: usize) -> String {
    match value {
        PhpValue::Str(string) => {
            b = escape_string(&string, b);
        }
        PhpValue::Int(num) => {
            b.push_str(&num.to_string());
        }
        PhpValue::Bool(bl) => {
            b.push_str(if bl { "T" } else { "F" });
        }
        PhpValue::Arr(map) => {
            // Represent array as json array with
            // tuple arrays [key, value] inside
            // This preserves int keys from php
            b.push_str("(");
            for (k, v) in map {
                b.push('\n');
                for _ in 0..indent {
                    b.push('\t');
                }
                b.push('(');
                match k {
                    ArrKey::Int(idx) => {
                        b.push_str(&idx.to_string());
                    }
                    ArrKey::Str(string) => {
                        b = escape_string(&string, b);
                    }
                }
                b.push(' ');
                b = serialize_php(b, v, indent + 1);
                b.push(')');
            }
            b.push('\n');
            if indent > 1 {
                for _ in 0..indent - 1 {
                    b.push('\t');
                }
            }
            b.push(')');
        }
        PhpValue::Null => {
            b.push_str("N");
        }
        PhpValue::Float(f) => {
            b.push_str(&f.to_string());
        }
        PhpValue::Object(class_name, members) => {
            // Both types of objects are encoded as
            // json objects with class_name as key,
            // standard serialization will have an
            // array of members, while serializable
            // will have a string
            b.push_str("(O ");
            b = escape_string(&class_name, b);
            b.push_str(" (");
            for (member_key, member_value) in members {
                b.push('\n');
                for _ in 0..indent {
                    b.push('\t');
                }
                b.push_str("(");
                b = escape_string(&member_key, b);
                b.push(' ');
                b = serialize_php(b, member_value, indent + 1);
                b.push(')');
            }
            b.push('\n');
            if indent > 1 {
                for _ in 0..indent - 1 {
                    b.push('\t');
                }
            }
            b.push(')');
            b.push(')');
        }
        PhpValue::Serializable(class_name, serialized) => {
            b.push_str("(C ");
            b = escape_string(&class_name, b);
            b.push(' ');
            b = escape_string(&serialized, b);
            b.push(')');
        }
    }
    b
}

pub fn escape_string(s: &str, mut b: String) -> String {
    b.push('"');
    for chr in s.chars() {
        // Similar to char.escape_default()
        match chr {
            '"' => b.push_str("\\\""),
            // Expecting \n to be present as well
            '\r' => (),
            '\\' => b.push_str("\\\\"),
            other => b.push(other)
        }
    }
    b.push('"');
    b
}
