pub fn json_escape_string(s: &str, mut b: String) -> String {
    b.push('"');
    for chr in s.chars() {
        // Similar to char.escape_default()
        match chr {
            '"' => b.push_str("\\\""),
            '\n' => b.push_str("\\n"),
            '\t' => b.push_str("\\t"),
            // Expecting \n to be present as well
            '\r' => (),
            '\\' => b.push_str("\\\\"),
            non_uni @ '\x20' ..= '\x7e' => b.push(non_uni),
            other => {
                // from char.escape_unicode();
                let c = other as u32;
                let msb = 31 - (c | 1).leading_zeros();
                let ms_hex_digit = msb / 4;
                b.push_str("\\u");
                // Fill
                if ms_hex_digit < 3 {
                    for _ in 1..=(3 - ms_hex_digit) {
                        b.push('0');
                    }
                }
                for hex_digit_idx in (0..=ms_hex_digit).rev() {
                    let hex_digit = ((c) >> (hex_digit_idx * 4)) & 0xf;
                    b.push(std::char::from_digit(hex_digit, 16).unwrap());
                }
            }
        }
    }
    b.push('"');
    b
}
