#[must_use]
pub fn escape_terminal_text(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());

    for ch in input.chars() {
        match ch {
            '\x1b' => escaped.push_str("\\x1b"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_control() => {
                escaped.push_str("\\x");
                let code = ch as u32;
                escaped.push(hex_digit(((code >> 4) & 0x0f) as u8));
                escaped.push(hex_digit((code & 0x0f) as u8));
            }
            ch => escaped.push(ch),
        }
    }

    escaped
}

fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => (b'a' + value - 10) as char,
        _ => '0',
    }
}
