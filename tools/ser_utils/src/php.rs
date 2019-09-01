use indexmap::IndexMap;

// For Arr, we could use an IndexMap when
// needing to query, but a vec is a better
// fit for succint representation.
// For optimizations for example when
// writing json output, we could also consider
// a more direct approach to json writing
#[derive(Clone, Debug, PartialEq)]
pub enum PhpValue {
    Arr(IndexMap<ArrKey, PhpValue>),
    Str(String),
    Int(i64),
    Bool(bool),
    Float(f64),
    Null,
    // Default object serialization
    // (class_name, members)
    // Key will have some special characters
    // to signify public, protected or private
    Object(String, IndexMap<String, PhpValue>),
    // Object serialized by
    // custom serialization function
    // (class_name, serialized)
    Serializable(String, String)
}

// Todo: Test if these are all possibilities
// from php
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ArrKey {
    Str(String),
    Int(i64),
}

// http://www.phpinternalsbook.com/php5/classes_objects/serialization.html

pub fn deserialize(s: &[u8]) -> Result<PhpValue, String> {
    let (_rest, php_value) = deserialize_value(s)?;
    Ok(php_value)
}

// Todo: Consider some "query dsl", if need arises
// one example: https://www.reddit.com/r/rust/comments/cl3sp7/gjson_json_parser_for_rust_get_json_values_quickly/

pub fn serialize(value: PhpValue, mut s: String) -> String {
    match value {
        PhpValue::Str(string) => {
            s.push_str(&format!("s:{}:\"{}\";", string.len(), string));
            s
        }
        PhpValue::Int(num) => {
            s.push_str(&format!("i:{};", num));
            s
        }
        PhpValue::Bool(b) => {
            s.push_str(&format!("b:{};", if b {'1'} else {'0'}));
            s
        }
        PhpValue::Arr(map) => {
            s.push_str(&format!("a:{}:{{", map.len()));
            for (k, v) in map {
                match k {
                    ArrKey::Int(idx) => {
                        s.push_str(&format!("i:{};", idx));
                    }
                    ArrKey::Str(string) => {
                        s.push_str(&format!("s:{}:\"{}\";", string.len(), string));
                    }
                }
                s = serialize(v, s);
            }
            s.push('}');
            s
        }
        PhpValue::Null => {
            s.push_str("N;");
            s
        }
        PhpValue::Float(f) => {
            s.push_str(&format!("d:{};", f));
            s
        }
        PhpValue::Object(class_name, members) => {
            s.push_str("O:");
            s.push_str(&class_name.len().to_string());
            s.push_str(":\"");
            s.push_str(&class_name);
            s.push_str("\":");
            s.push_str(&members.len().to_string());
            s.push_str(":{");
            for (name, value) in members {
                s = serialize(PhpValue::Str(name), s);
                s = serialize(value, s);
            }
            s.push('}');
            s
        }
        PhpValue::Serializable(class_name, serialized) => {
            s.push_str("C:");
            s.push_str(&class_name.len().to_string());
            s.push_str(":\"");
            s.push_str(&class_name);
            s.push_str("\":");
            s.push_str(&serialized.len().to_string());
            s.push_str(":{");
            s.push_str(&serialized);
            s.push('}');
            s
        }
    }
}
const A_U8: u8 = 'a' as u8;
const I_U8: u8 = 'i' as u8;
const S_U8: u8 = 's' as u8;
const B_U8: u8 = 'b' as u8;
const D_U8: u8 = 'd' as u8;
const N_U8: u8 = 'N' as u8;
const O_U8: u8 = 'O' as u8;
const C_U8: u8 = 'C' as u8;

fn deserialize_value(s: &[u8]) -> Result<(&[u8], PhpValue), String> {
    if s.len() < 1 {
        return Err("Could not deserialize value from empty string".to_string());
    }
    match s[0] {
        A_U8 => {
            // Assoc or array
            let s = expect_char(&s[1..], ':')?;
            let (s, elements) = read_number(s)?;
            // a:xx:{x
            let s = expect_char(s, ':')?;
            let mut s = expect_char(s, '{')?;
            // Elements
            let mut map = IndexMap::new();
            for _i in 0..elements {
                let (s2, (key, value)) = read_pair(s)?;
                match key {
                    PhpValue::Int(idx) => {
                        map.insert(ArrKey::Int(idx), value);
                    }
                    PhpValue::Str(string) => {
                        map.insert(ArrKey::Str(string), value);
                    }
                    other => return Err(format!("Expected int or string as key, got: {:?}", other)),
                }
                s = s2;
            }
            let s = expect_char(s, '}')?;
            Ok((s, PhpValue::Arr(map)))
        }
        I_U8 => {
            // Parse int
            let s = expect_char(&s[1..], ':')?;
            let (s, num) = read_number(s)?;
            let s = expect_char(s, ';')?;
            Ok((s, PhpValue::Int(num)))
        }
        S_U8 => {
            // Parse string
            let s = expect_char(&s[1..], ':')?;
            let (s, len) = read_number(s)?;
            let s = expect_char(s, ':')?;
            let (s, string) = read_string(s, len as usize)?;
            let s = expect_char(s, ';')?;
            Ok((s, PhpValue::Str(string)))
        }
        B_U8 => {
            // Bool
            let s = expect_char(&s[1..], ':')?;
            // Shortcut checking for also for ';'
            if s.len() < 2 || s[1] != ';' as u8 {
                return Err("Error parsing boolean".to_string());
            }
            if s[2] == '1' as u8 {
                Ok((&s[2..], PhpValue::Bool(true)))
            } else {
                // Assuming '0', false
                Ok((&s[2..], PhpValue::Bool(false)))
            }
        }
        D_U8 => {
            let s = expect_char(&s[1..], ':')?;
            let (s, whole) = read_number(s)?;
            let s = expect_char(s, '.')?;
            let (s, fraction) = read_number(s)?;
            // Slightly suboptimal
            let f = match format!("{}.{}", whole, fraction).parse::<f64>() {
                Ok(f) => f,
                Err(e) => return Err(format!("Error parsing f64, {:?}", e))
            };
            let s = expect_char(s, ';')?;
            Ok((s, PhpValue::Float(f)))
        }
        N_U8 => {
            let s = expect_char(&s[1..], ';')?;
            Ok((s, PhpValue::Null))
        }
        O_U8 => {
            // Serialized object
            let s = expect_char(&s[1..], ':')?;
            let (s, len) = read_number(s)?;
            let s = expect_char(s, ':')?;
            let (s, class_name) = read_string(s, len as usize)?;
            let s = expect_char(s, ':')?;
            let (s, num_members) = read_number(s)?;
            let s = expect_char(s, ':')?;
            let mut s = expect_char(s, '{')?;
            // Members with values
            let mut map = IndexMap::new();
            for _i in 0..num_members {
                // Could optimize a bit with custom member
                // part reading
                let (s2, (member, value)) = read_pair(s)?;
                match member {
                    PhpValue::Str(string) => {
                        map.insert(string, value);
                    }
                    other => return Err(format!("Expected string as member, got: {:?}", other)),
                }
                s = s2;
            }
            let s = expect_char(s, '}')?;
            Ok((s, PhpValue::Object(class_name, map)))
        }
        C_U8 => {
            let s = expect_char(&s[1..], ':')?;
            let (s, len) = read_number(s)?;
            let s = expect_char(s, ':')?;
            let (s, class_name) = read_string(s, len as usize)?;
            let s = expect_char(s, ':')?;
            let (s, len) = read_number(s)?;
            let len = len as usize;
            let s = expect_char(s, ':')?;
            let s = expect_char(s, '{')?;
            if s.len() < len {
                return Err(format!("Input length less than given length: {}", len));
            }
            let serialized = String::from_utf8_lossy(&s[..len]).to_string();
            let s = expect_char(&s[len..], '}')?;
            Ok((s, PhpValue::Serializable(class_name, serialized)))
        }
        other => {
            //println!("{}", String::from_utf8_lossy(s));
            return Err(format!("Unexpected char: {}", other as char))
        },
    }
}

const N1: u8 = '1' as u8;
const N2: u8 = '2' as u8;
const N3: u8 = '3' as u8;
const N4: u8 = '4' as u8;
const N5: u8 = '5' as u8;
const N6: u8 = '6' as u8;
const N7: u8 = '7' as u8;
const N8: u8 = '8' as u8;
const N9: u8 = '9' as u8;
const N0: u8 = '0' as u8;
fn read_number(s: &[u8]) -> Result<(&[u8], i64), String> {
    let mut n: i64 = 0;
    let mut i = 0;
    let len = s.len();
    let (neg, s) = if len > 0 && s[0] == '-' as u8 {
        (true, &s[1..])
    } else {
        (false, s)
    };
    while i < len {
        n = match s[i] {
            N1 => n * 10 + 1,
            N2 => n * 10 + 2,
            N3 => n * 10 + 3,
            N4 => n * 10 + 4,
            N5 => n * 10 + 5,
            N6 => n * 10 + 6,
            N7 => n * 10 + 7,
            N8 => n * 10 + 8,
            N9 => n * 10 + 9,
            N0 => n * 10,
            _ => break
        };
        i = i + 1;
    }
    // Return read number and number of digits
    if i == 0 {
        println!("{}", String::from_utf8_lossy(s));
        Err("No digits found".to_string())
    } else {
        Ok((&s[i..], if neg { n * -1 } else { n }))
    }
}
#[inline]
fn read_pair(s: &[u8]) -> Result<(&[u8], (PhpValue, PhpValue)), String> {
    let (s, first) = deserialize_value(s)?;
    let (s, second) = deserialize_value(s)?;
    Ok((s, (first, second)))
}
// Given char is tested as u8
// We don't want to be too optimistic in this
// lib as it's used to check if strings are
// php serialized by attempting to parse them
// If we had been sure, we could skip most of these
#[inline]
fn expect_char(s: &[u8], c: char) -> Result<&[u8], String> {
    if s.len() > 0 && s[0] == c as u8 {
        Ok(&s[1..])
    } else {
        Err(format!("Expected char not found: {}", c))
    }
}

/// Reads a quotes string
#[inline]
fn read_string(s: &[u8], len: usize) -> Result<(&[u8], String), String> {
    let s = expect_char(s, '"')?;
    if s.len() < len {
        return Err(format!("Input length less than given length: {}", len));
    }
    // Could consider borrow/Cow
    let string = String::from_utf8_lossy(&s[..len]).to_string();
    let s = expect_char(&s[len..], '"')?;
    Ok((s, string))
}