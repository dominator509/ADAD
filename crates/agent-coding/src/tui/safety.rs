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

/// Sanitize untrusted model/tool text for one-shot CLI printing. Terminal
/// control sequences (notably ESC) are neutralized while line breaks, carriage
/// returns, and tabs are preserved so piped output stays usable.
#[must_use]
pub fn sanitize_cli_text(input: &str) -> String {
    let mut sanitized = String::with_capacity(input.len());

    for ch in input.chars() {
        match ch {
            '\n' | '\r' | '\t' => sanitized.push(ch),
            ch if ch.is_control() => {
                sanitized.push_str("\\x");
                let code = ch as u32;
                sanitized.push(hex_digit(((code >> 4) & 0x0f) as u8));
                sanitized.push(hex_digit((code & 0x0f) as u8));
            }
            ch => sanitized.push(ch),
        }
    }

    sanitized
}

#[cfg(test)]
mod tests {
    use super::{escape_terminal_text, sanitize_cli_text};

    #[test]
    fn cli_sanitizer_neutralizes_escape_sequences_but_keeps_line_breaks() {
        let input = "line one\nline two\x1b]0;pwned\x07\ttabbed\x1b[2J";

        let sanitized = sanitize_cli_text(input);

        assert!(!sanitized.contains('\x1b'));
        assert!(!sanitized.contains('\x07'));
        assert!(sanitized.contains('\n'));
        assert!(sanitized.contains('\t'));
        assert!(sanitized.contains("line one"));
    }

    #[test]
    fn cli_sanitizer_leaves_plain_text_untouched() {
        let input = "The answer is 42.\nSee https://example.invalid for details.";

        assert_eq!(sanitize_cli_text(input), input);
    }

    #[test]
    fn terminal_escaper_still_covers_all_controls() {
        assert_eq!(escape_terminal_text("a\x1bb\nc"), "a\\x1bb\\nc");
    }
}
