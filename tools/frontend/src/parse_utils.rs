/// Extension to byte arrays aiding parsing

pub trait ByteArrayExt<'a> {
    fn chr(self) -> u8;
    fn parse_chr(self, c: u8) -> Option<&'a [u8]>;
    fn parse_chr_opt_space(self, c: u8) -> Option<&'a [u8]>;
    fn raw(self, idx: usize) -> u8;
    fn is(self, other: u8) -> bool;
    fn is_uc(self) -> bool;
    fn is_ident(self) -> bool;
    fn parse_bstr(self, bstr: &[u8]) -> Option<&'a [u8]>;
    fn parse_bstr_opt_space(self, bstr: &[u8]) -> Option<&'a [u8]>;
    fn parse_ident(self, bstr: &[u8]) -> Option<&'a [u8]>;
    fn parse_ident1(self, chr: u8) -> Option<&'a [u8]>;
    // Maybe not much point to putting in array, should
    // probably be as well optimized, but could change to args
    fn parse_ident2(self, rest: [u8; 2]) -> Option<&'a [u8]>;
    fn parse_ident3(self, rest: [u8; 3]) -> Option<&'a [u8]>;
    fn parse_ident4(self, rest: [u8; 4]) -> Option<&'a [u8]>;
    fn parse_ident5(self, rest: [u8; 5]) -> Option<&'a [u8]>;
    fn parse_ident6(self, rest: [u8; 6]) -> Option<&'a [u8]>;
    fn close_tag1(self, a: u8) -> Option<&'a [u8]>;
    fn close_tag2(self, a: u8, b: u8) -> Option<&'a [u8]>;
    fn close_tag3(self, a: u8, b: u8, c: u8) -> Option<&'a [u8]>;
    fn close_tag4(self, a: u8, b: u8, c: u8, d: u8) -> Option<&'a [u8]>;
    fn close_tag5(self, a: u8, b: u8, c: u8, d: u8, e: u8) -> Option<&'a [u8]>;
    fn close_tag6(self, a: u8, b: u8, c: u8, d: u8, e: u8, f: u8) -> Option<&'a [u8]>;
    fn close_tag7(self, a: u8, b: u8, c: u8, d: u8, e: u8, f: u8, g: u8) -> Option<&'a [u8]>;
    fn len_gt(self, l: usize) -> Result<()>;
}

// I wonder if we could incorporate some length
// checks in the type system.
// Especially "has one" is a easier candidate
// If not for performance then for some ease of
// mind
// Branching on "opt_space"'d maybe prove frustrating
// "const generics" might be relevant

pub type Result<T> = std::result::Result<T, ParseError>;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    // (msg, around)
    Msg(String, String),
}
pub fn err<T, S: Into<String>>(msg: S, s: &[u8]) -> Result<T> {
    Err(ParseError::Msg(msg.into(), ParseError::around(s)))
}
impl ParseError {
    fn around(s: &[u8]) -> String {
        const MAX: usize = 100;
        if s.len() >= MAX {
            String::from_utf8_lossy(&s[..MAX]).into()
        } else {
            String::from_utf8_lossy(s).into()
        }
    }
}
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseError::Msg(msg, around) => write!(f, "{}, around: {}", msg, around),
        }
    }
}

impl<'a> ByteArrayExt<'a> for &'a [u8] {
    #[inline]
    fn chr(self) -> u8 {
        unsafe { *self.as_ptr() }
    }

    #[inline]
    fn parse_chr(self, c: u8) -> Option<&'a [u8]> {
        if unsafe { *self.as_ptr() } == c {
            Some(&self[1..])
        } else {
            None
        }
    }

    #[inline]
    fn parse_chr_opt_space(self, c: u8) -> Option<&'a [u8]> {
        if self.len() > 0 && self.chr() == c {
            Some(&self[1..])
        } else {
            let s = strip_space(self);
            if s.len() > 0 && s.chr() == c {
                Some(&s[1..])
            } else {
                None
            }
        }
    }

    #[inline]
    fn raw(self, idx: usize) -> u8 {
        unsafe { *self.as_ptr().add(idx) }
    }

    #[inline]
    fn is(self, other: u8) -> bool {
        self.raw(0) == other
    }

    #[inline]
    fn is_uc(self) -> bool {
        let v = self.raw(0);
        v >= 65 && v <= 90
    }

    #[inline]
    fn is_ident(self) -> bool {
        is_ident_char(self.chr())
    }

    #[inline]
    fn parse_bstr(self, bstr: &[u8]) -> Option<&'a [u8]> {
        let len = self.len();
        let blen = bstr.len();
        if len < blen {
            None
        } else {
            for i in 0..blen {
                if self.raw(i) != bstr.raw(i) {
                    return None;
                }
            }
            Some(&self[blen..])
        }
    }

    /// Parse bstr when string is likely, space allowed
    #[inline]
    fn parse_bstr_opt_space(self, bstr: &[u8]) -> Option<&'a [u8]> {
        let len = self.len();
        let blen = bstr.len();
        if len < blen {
            None
        } else {
            for i in 0..blen {
                if self.raw(i) != bstr.raw(i) {
                    if i == 0 && is_space(self.chr()) {
                        let s = strip_space(&self[1..]);
                        return s.parse_bstr(bstr);
                    } else {
                        return None;
                    }
                }
            }
            Some(&self[blen..])
        }
    }

    /// Attempts to parse given bstr, and then
    /// a non-ident char
    /// I slightly more optimized version could
    /// know the part of bstr that succeded,
    /// but this seems a rearer case, skipping
    /// in interest of code simplicity
    #[inline]
    fn parse_ident(self, bstr: &[u8]) -> Option<&'a [u8]> {
        let len = self.len();
        let blen = bstr.len();
        if len < blen + 1 {
            None
        } else {
            for i in 0..blen {
                if self.raw(i) != bstr.raw(i) {
                    return None;
                }
            }
            if is_ident_char(self.raw(blen + 1)) {
                // Ident not ended
                None
            } else {
                Some(&self[blen..])
            }
        }
    }

    // Special case for when the ident is only one more char
    #[inline]
    fn parse_ident1(self, chr: u8) -> Option<&'a [u8]> {
        if self.len() < 2 {
            None
        } else if self.chr() == chr && !is_ident_char(self.raw(1)) {
            Some(&self[1..])
        } else {
            None
        }
    }

    #[inline]
    fn parse_ident2(self, rest: [u8; 2]) -> Option<&'a [u8]> {
        if self.len() < 3 {
            None
        } else if self.chr() == rest[0] && self.raw(1) == rest[1] && !is_ident_char(self.raw(2)) {
            Some(&self[2..])
        } else {
            None
        }
    }

    #[inline]
    fn parse_ident3(self, rest: [u8; 3]) -> Option<&'a [u8]> {
        if self.len() < 4 {
            None
        } else if self.chr() == rest[0]
            && self.raw(1) == rest[1]
            && self.raw(2) == rest[2]
            && !is_ident_char(self.raw(3))
        {
            Some(&self[3..])
        } else {
            None
        }
    }

    #[inline]
    fn parse_ident4(self, rest: [u8; 4]) -> Option<&'a [u8]> {
        if self.len() < 5 {
            None
        } else if self.chr() == rest[0]
            && self.raw(1) == rest[1]
            && self.raw(2) == rest[2]
            && self.raw(3) == rest[3]
            && !is_ident_char(self.raw(4))
        {
            Some(&self[4..])
        } else {
            None
        }
    }

    #[inline]
    fn parse_ident5(self, rest: [u8; 5]) -> Option<&'a [u8]> {
        if self.len() < 6 {
            None
        } else if self.chr() == rest[0]
            && self.raw(1) == rest[1]
            && self.raw(2) == rest[2]
            && self.raw(3) == rest[3]
            && self.raw(4) == rest[4]
            && !is_ident_char(self.raw(5))
        {
            Some(&self[5..])
        } else {
            None
        }
    }

    #[inline]
    fn parse_ident6(self, rest: [u8; 6]) -> Option<&'a [u8]> {
        if self.len() < 7 {
            None
        } else if self.chr() == rest[0]
            && self.raw(1) == rest[1]
            && self.raw(2) == rest[2]
            && self.raw(3) == rest[3]
            && self.raw(4) == rest[4]
            && self.raw(5) == rest[5]
            && !is_ident_char(self.raw(6))
        {
            Some(&self[6..])
        } else {
            None
        }
    }

    /// These parse close tag from after </
    #[inline]
    fn close_tag1(self, a: u8) -> Option<&'a [u8]> {
        if let Some(s) = self.parse_chr_opt_space(a) {
            s.parse_chr_opt_space(RBRACKET)
        } else {
            None
        }
    }
    fn close_tag2(self, a: u8, b: u8) -> Option<&'a [u8]> {
        if let Some(s) = self.parse_chr_opt_space(a) {
            if s.len() < 2 {
                None
            } else if s.chr() == b {
                s[1..].parse_chr_opt_space(RBRACKET)
            } else {
                None
            }
        } else {
            None
        }
    }
    fn close_tag3(self, a: u8, b: u8, c: u8) -> Option<&'a [u8]> {
        if let Some(s) = self.parse_chr_opt_space(a) {
            if s.len() < 3 {
                None
            } else if s.chr() == b && s.raw(1) == c {
                s[2..].parse_chr_opt_space(RBRACKET)
            } else {
                None
            }
        } else {
            None
        }
    }
    fn close_tag4(self, a: u8, b: u8, c: u8, d: u8) -> Option<&'a [u8]> {
        if let Some(s) = self.parse_chr_opt_space(a) {
            if s.len() < 4 {
                None
            } else if s.chr() == b && s.raw(1) == c && s.raw(2) == d {
                s[3..].parse_chr_opt_space(RBRACKET)
            } else {
                None
            }
        } else {
            None
        }
    }
    fn close_tag5(self, a: u8, b: u8, c: u8, d: u8, e: u8) -> Option<&'a [u8]> {
        if let Some(s) = self.parse_chr_opt_space(a) {
            if s.len() < 5 {
                None
            } else if s.chr() == b && s.raw(1) == c && s.raw(2) == d && s.raw(3) == e {
                s[4..].parse_chr_opt_space(RBRACKET)
            } else {
                None
            }
        } else {
            None
        }
    }
    fn close_tag6(self, a: u8, b: u8, c: u8, d: u8, e: u8, f: u8) -> Option<&'a [u8]> {
        if let Some(s) = self.parse_chr_opt_space(a) {
            if s.len() < 6 {
                None
            } else if s.chr() == b
                && s.raw(1) == c
                && s.raw(2) == d
                && s.raw(3) == e
                && s.raw(4) == f
            {
                s[5..].parse_chr_opt_space(RBRACKET)
            } else {
                None
            }
        } else {
            None
        }
    }
    fn close_tag7(self, a: u8, b: u8, c: u8, d: u8, e: u8, f: u8, g: u8) -> Option<&'a [u8]> {
        if let Some(s) = self.parse_chr_opt_space(a) {
            if s.len() < 7 {
                None
            } else if s.chr() == b
                && s.raw(1) == c
                && s.raw(2) == d
                && s.raw(3) == e
                && s.raw(4) == f
                && s.raw(5) == g
            {
                s[6..].parse_chr_opt_space(RBRACKET)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Requires len to be greater than given usize
    #[inline]
    fn len_gt(self, l: usize) -> Result<()> {
        if self.len() > l {
            Ok(())
        } else {
            err(format!("Required length to be greater than {}", l), self)
        }
    }
}

// For significant whitespace (type one or more)
// we could have a wrapper functions that detect
#[inline]
pub fn strip_space(s: &[u8]) -> &[u8] {
    let len = s.len();
    if len == 0 || !is_space(s.chr()) {
        s
    } else {
        let mut i = 1;
        while i < len {
            if is_space(s.raw(i)) {
                i += 1;
            } else {
                return &s[i..];
            }
        }
        // Only spaces or empty slice
        &s[len..]
    }
}

#[inline]
pub fn is_space(c: u8) -> bool {
    c == SPACE || c == TAB || c == NL || c == CR
}

/// Check if first byte is in given range
/// Assumes len() > 0
#[inline]
pub fn in_range(s: u8, min: u8, max: u8) -> bool {
    s >= min && s <= max
}

#[inline]
pub fn is_alpha(s: u8) -> bool {
    in_range(s, 97, 122) || in_range(s, 65, 90)
}

#[inline]
pub fn is_numeric(s: u8) -> bool {
    in_range(s, 48, 57)
}

#[inline]
pub fn is_alphanum(s: u8) -> bool {
    // Checking lowercase, then numbers
    in_range(s, 97, 122) || in_range(s, 48, 57) || in_range(s, 65, 90)
}

#[inline]
pub fn is_ident_char(c: u8) -> bool {
    is_alphanum(c) || c == DASH
}

/// Small helper to create a string with some
/// initial content with a given capacity
#[inline]
pub fn sbuf(from: &str, capacity: usize) -> String {
    let mut b = String::with_capacity(capacity);
    b.push_str(from);
    b
}

/// Small helper to create a string with some
/// initial content with a given capacity
#[inline]
pub fn sbuf_chr(from: char, capacity: usize) -> String {
    let mut b = String::with_capacity(capacity);
    b.push(from);
    b
}

// not used
/// When uppercase char, converts to lower case
#[inline]
pub fn lc_char(c: u8) -> u8 {
    if c >= 65 && c <= 90 {
        c + 32
    } else {
        c
    }
}

pub const LBRACKET: u8 = '<' as u8;
pub const RBRACKET: u8 = '>' as u8;
pub const FSLASH: u8 = '/' as u8;
pub const BSLASH: u8 = '\\' as u8;
pub const EQUAL: u8 = '=' as u8;
pub const DQUOTE: u8 = '"' as u8;
pub const SQUOTE: u8 = '\'' as u8;
pub const DASH: u8 = '-' as u8;
pub const SPACE: u8 = ' ' as u8;
pub const TAB: u8 = '\t' as u8;
pub const NL: u8 = '\n' as u8;
pub const CR: u8 = '\r' as u8;
pub const EXCLAMATION: u8 = '!' as u8;

pub const N1: u8 = '1' as u8;
pub const N2: u8 = '2' as u8;
pub const N3: u8 = '3' as u8;
pub const N4: u8 = '4' as u8;
pub const N5: u8 = '5' as u8;
pub const N6: u8 = '6' as u8;
pub const N7: u8 = '7' as u8;
pub const N8: u8 = '8' as u8;
pub const N9: u8 = '9' as u8;
pub const N0: u8 = '0' as u8;

pub const A: u8 = 'a' as u8;
pub const B: u8 = 'b' as u8;
pub const C: u8 = 'c' as u8;
pub const D: u8 = 'd' as u8;
pub const E: u8 = 'e' as u8;
pub const F: u8 = 'f' as u8;
pub const G: u8 = 'g' as u8;
pub const H: u8 = 'h' as u8;
pub const I: u8 = 'i' as u8;
pub const J: u8 = 'j' as u8;
pub const K: u8 = 'k' as u8;
pub const L: u8 = 'l' as u8;
pub const M: u8 = 'm' as u8;
pub const N: u8 = 'n' as u8;
pub const O: u8 = 'o' as u8;
pub const P: u8 = 'p' as u8;
pub const Q: u8 = 'q' as u8;
pub const R: u8 = 'r' as u8;
pub const S: u8 = 's' as u8;
pub const T: u8 = 't' as u8;
pub const U: u8 = 'u' as u8;
pub const V: u8 = 'v' as u8;
pub const W: u8 = 'w' as u8;
pub const X: u8 = 'x' as u8;
pub const Y: u8 = 'y' as u8;
pub const Z: u8 = 'z' as u8;

pub const UC_A: u8 = 'A' as u8;
pub const UC_B: u8 = 'B' as u8;
pub const UC_C: u8 = 'C' as u8;
pub const UC_D: u8 = 'D' as u8;
pub const UC_E: u8 = 'E' as u8;
pub const UC_F: u8 = 'F' as u8;
pub const UC_G: u8 = 'G' as u8;
pub const UC_H: u8 = 'H' as u8;
pub const UC_I: u8 = 'I' as u8;
pub const UC_J: u8 = 'J' as u8;
pub const UC_K: u8 = 'K' as u8;
pub const UC_L: u8 = 'L' as u8;
pub const UC_M: u8 = 'M' as u8;
pub const UC_N: u8 = 'N' as u8;
pub const UC_O: u8 = 'O' as u8;
pub const UC_P: u8 = 'P' as u8;
pub const UC_Q: u8 = 'Q' as u8;
pub const UC_R: u8 = 'R' as u8;
pub const UC_S: u8 = 'S' as u8;
pub const UC_T: u8 = 'T' as u8;
pub const UC_U: u8 = 'U' as u8;
pub const UC_V: u8 = 'V' as u8;
pub const UC_W: u8 = 'W' as u8;
pub const UC_X: u8 = 'X' as u8;
pub const UC_Y: u8 = 'Y' as u8;
pub const UC_Z: u8 = 'Z' as u8;

// next_code_point experimental
// https://doc.rust-lang.org/src/core/str/mod.rs.html#500-528
/// Mask of the value bits of a continuation byte.
const CONT_MASK: u8 = 0b0011_1111;
#[inline]
fn utf8_acc_cont_byte(ch: u32, byte: u8) -> u32 {
    (ch << 6) | (byte & CONT_MASK) as u32
}
/// Next utf8 char from byte array
pub fn next_utf8(s: &[u8]) -> Option<(usize, char)> {
    let len = s.len();
    if len == 0 {
        return None;
    }
    // Decode UTF-8
    let x = s.chr();
    if x < 128 {
        return Some((1, x as char));
    }

    // Multibyte case follows
    let mut num_bytes = 1;
    // Decode from a byte combination out of: [[[x y] z] w]
    // NOTE: Performance is sensitive to the exact formulation here
    //utf8_first_byte
    let init = (x & (0x7F >> 2)) as u32;
    //let y = unwrap_or_0(bytes.next());
    let y = if len > 1 {
        num_bytes = 2;
        s.raw(1)
    } else {
        0
    };
    let mut ch = utf8_acc_cont_byte(init, y);
    if x >= 0xE0 {
        // [[x y z] w] case
        // 5th bit in 0xE0 .. 0xEF is always clear, so `init` is still valid
        //let z = unwrap_or_0(bytes.next());
        let z = if len > 2 {
            num_bytes = 3;
            s.raw(2)
        } else {
            0
        };
        let y_z = utf8_acc_cont_byte((y & CONT_MASK) as u32, z);
        ch = init << 12 | y_z;
        if x >= 0xF0 {
            // [x y z w] case
            // use only the lower 3 bits of `init`
            //let w = unwrap_or_0(bytes.next());
            let w = if len > 3 {
                num_bytes = 4;
                s.raw(3)
            } else {
                0
            };
            ch = (init & 7) << 18 | utf8_acc_cont_byte(y_z, w);
        }
    }
    Some((num_bytes, unsafe { std::char::from_u32_unchecked(ch) }))
}
