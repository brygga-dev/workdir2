use crate::php::{PhpValue, ArrKey};
use crate::json::json_escape_string;

// Meant (at least) for git stored data
// Increases single line diffs, making changes
// easier to read, and reduces diff size
pub fn serialize_json_lines(mut b: String, value: PhpValue, indent: usize) -> String {
    match value {
        PhpValue::Str(string) => {
            b = json_escape_string(&string, b);
        }
        PhpValue::Int(num) => {
            b.push_str(&num.to_string());
        }
        PhpValue::Bool(bl) => {
            b.push_str(if bl { "true" } else { "false" });
        }
        PhpValue::Arr(map) => {
            // Represent array as json array with
            // tuple arrays [key, value] inside
            // This preserves int keys from php
            b.push_str("[");
            let mut curr = 0;
            for (k, v) in map {
                if curr > 0 {
                    b.push_str(",\n");
                } else {
                    b.push('\n');
                }
                for _ in 0..indent {
                    b.push('\t');
                }
                b.push_str("[");
                match k {
                    ArrKey::Int(idx) => {
                        b.push_str(&idx.to_string());
                    }
                    ArrKey::Str(string) => {
                        b = json_escape_string(&string, b);
                    }
                }
                b.push_str(", ");
                b = serialize_json_lines(b, v, indent + 1);
                b.push(']');
                curr += 1;
            }
            b.push('\n');
            if indent > 1 {
                for _ in 0..indent - 1 {
                    b.push('\t');
                }
            }
            b.push(']');
        }
        PhpValue::Null => {
            b.push_str("null");
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
            b.push('{');
            b = json_escape_string(&class_name, b);
            b.push_str(": [");
            let mut curr = 0;
            for (member_key, member_value) in members {
                if curr > 0 {
                    b.push_str(",\n");
                } else {
                    b.push('\n');
                }
                for _ in 0..indent {
                    b.push('\t');
                }
                b.push_str("[");
                b = json_escape_string(&member_key, b);
                b.push_str(", ");
                b = serialize_json_lines(b, member_value, indent + 1);
                b.push(']');
                curr += 1;
            }
            b.push('\n');
            if indent > 1 {
                for _ in 0..indent - 1 {
                    b.push('\t');
                }
            }
            b.push(']');
            b.push('}');
        }
        PhpValue::Serializable(class_name, serialized) => {
            b.push('{');
            b = json_escape_string(&class_name, b);
            b.push_str(": ");
            b = json_escape_string(&serialized, b);
            b.push('}');
        }
    }
    b
}