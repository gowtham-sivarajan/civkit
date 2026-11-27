// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 15:40 IST · last edited 2026-11-27 15:40 IST

//! Decoding DXF strings (ADR 0015, Q-C): UTF-8 from R2007 (AC1021) on; before that, the `$DWGCODEPAGE` the header
//! states: `ANSI_1252` is decoded, any other (or none) is not guessed: its non-ASCII bytes become U+FFFD and a note
//! says so. `\U+XXXX` escapes are decoded in every version. Decoding runs twice over a value (measure, then write),
//! so the exact length can be reserved before anything is allocated.

/// How a file's strings are decoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Encoding {
    /// R2007 and later.
    Utf8,
    /// Before R2007, `$DWGCODEPAGE = ANSI_1252`.
    Ansi1252,
    /// Before R2007 with another code page, or none stated: only ASCII is decoded.
    AsciiOnly,
}

/// Windows-1252 bytes 0x80–0x9F; `None` where the code page defines no character.
const CP1252_HIGH: [Option<char>; 32] = [
    Some('\u{20AC}'),
    None,
    Some('\u{201A}'),
    Some('\u{0192}'),
    Some('\u{201E}'),
    Some('\u{2026}'),
    Some('\u{2020}'),
    Some('\u{2021}'),
    Some('\u{02C6}'),
    Some('\u{2030}'),
    Some('\u{0160}'),
    Some('\u{2039}'),
    Some('\u{0152}'),
    None,
    Some('\u{017D}'),
    None,
    None,
    Some('\u{2018}'),
    Some('\u{2019}'),
    Some('\u{201C}'),
    Some('\u{201D}'),
    Some('\u{2022}'),
    Some('\u{2013}'),
    Some('\u{2014}'),
    Some('\u{02DC}'),
    Some('\u{2122}'),
    Some('\u{0161}'),
    Some('\u{203A}'),
    Some('\u{0153}'),
    None,
    Some('\u{017E}'),
    Some('\u{0178}'),
];

/// A `\U+XXXX` escape at the start of `b`.
fn escape(b: &[u8]) -> Option<char> {
    let hex = b.strip_prefix(b"\\U+")?.get(..4)?;
    let text = std::str::from_utf8(hex).ok()?;
    char::from_u32(u32::from_str_radix(text, 16).ok()?)
}

/// The UTF-8 character at the start of `b`, and its length; `None` if `b` does not start with a valid one.
fn utf8_char(b: &[u8]) -> Option<(char, usize)> {
    let len = match b.first()? {
        0x00..=0x7F => 1,
        0xC2..=0xDF => 2,
        0xE0..=0xEF => 3,
        0xF0..=0xF4 => 4,
        _ => return None,
    };
    let s = std::str::from_utf8(b.get(..len)?).ok()?;
    Some((s.chars().next()?, len))
}

/// Decodes `bytes`, calling `out` for each character. Returns whether anything had to be replaced with U+FFFD.
pub(crate) fn decode(bytes: &[u8], encoding: Encoding, mut out: impl FnMut(char)) -> bool {
    let mut replaced = false;
    let mut i = 0;
    while i < bytes.len() {
        let rest = &bytes[i..];
        if let Some(c) = escape(rest) {
            out(c);
            i += 7;
            continue;
        }
        let b = rest[0];
        if b < 0x80 {
            out(char::from(b));
            i += 1;
            continue;
        }
        let decoded = match encoding {
            Encoding::Utf8 => utf8_char(rest),
            Encoding::Ansi1252 => match b {
                0x80..=0x9F => CP1252_HIGH[usize::from(b - 0x80)].map(|c| (c, 1)),
                _ => Some((char::from(b), 1)),
            },
            Encoding::AsciiOnly => None,
        };
        match decoded {
            Some((c, len)) => {
                out(c);
                i += len;
            }
            None => {
                out('\u{FFFD}');
                replaced = true;
                i += 1;
            }
        }
    }
    replaced
}

/// The decoded length of `bytes` in UTF-8, and whether anything would be replaced.
pub(crate) fn decoded_len(bytes: &[u8], encoding: Encoding) -> (usize, bool) {
    let mut len = 0;
    let replaced = decode(bytes, encoding, |c| len += c.len_utf8());
    (len, replaced)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(bytes: &[u8], e: Encoding) -> (String, bool) {
        let mut out = String::new();
        let r = decode(bytes, e, |c| out.push(c));
        (out, r)
    }

    #[test]
    fn each_encoding() {
        assert_eq!(s("café".as_bytes(), Encoding::Utf8), ("café".into(), false));
        assert_eq!(s(b"caf\xE9", Encoding::Utf8), ("caf\u{FFFD}".into(), true));
        assert_eq!(
            s(b"caf\xE9 \x80", Encoding::Ansi1252),
            ("café €".into(), false)
        );
        assert_eq!(s(b"\x81", Encoding::Ansi1252), ("\u{FFFD}".into(), true));
        assert_eq!(s(b"a\xE9", Encoding::AsciiOnly), ("a\u{FFFD}".into(), true));
        assert_eq!(
            s(b"\\U+00B0C \\U+zz", Encoding::AsciiOnly),
            ("°C \\U+zz".into(), false)
        );
        assert_eq!(decoded_len(b"caf\xE9", Encoding::Ansi1252), (5, false));
    }
}
