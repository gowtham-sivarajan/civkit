// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 15:40 IST · last edited 2026-11-27 15:40 IST

//! ASCII DXF as group-code / value pairs, read in place from the input bytes: no copy of the file is made, and
//! numbers are parsed from the value's bytes with the standard library's correctly rounded parser.

use crate::ReadError;

/// One group code and its value. `line` is the value's line (1-based).
#[derive(Debug, Clone, Copy)]
pub(crate) struct Pair<'a> {
    pub(crate) code: i32,
    pub(crate) value: &'a [u8],
    pub(crate) line: u64,
}

impl<'a> Pair<'a> {
    /// The value without surrounding ASCII whitespace.
    pub(crate) fn trimmed(&self) -> &'a [u8] {
        self.value.trim_ascii()
    }

    /// Whether the value, trimmed, is `word`.
    pub(crate) fn is(&self, word: &str) -> bool {
        self.trimmed() == word.as_bytes()
    }

    /// The value as a finite `f64`.
    pub(crate) fn f64(&self) -> Result<f64, ReadError> {
        let invalid = ReadError::InvalidNumber {
            line: self.line,
            code: self.code,
        };
        let text = std::str::from_utf8(self.trimmed()).map_err(|_| invalid.clone())?;
        let v: f64 = text.parse().map_err(|_| invalid)?;
        if v.is_finite() {
            Ok(v)
        } else {
            Err(ReadError::NonFinite {
                line: self.line,
                code: self.code,
            })
        }
    }

    /// The value as an integer that fits `T`.
    pub(crate) fn int<T: TryFrom<i64>>(&self) -> Result<T, ReadError> {
        let invalid = || ReadError::InvalidNumber {
            line: self.line,
            code: self.code,
        };
        let text = std::str::from_utf8(self.trimmed()).map_err(|_| invalid())?;
        let v: i64 = text.parse().map_err(|_| invalid())?;
        T::try_from(v).map_err(|_| invalid())
    }

    /// The value as a hexadecimal handle, if it is one.
    pub(crate) fn handle(&self) -> Option<u64> {
        let text = std::str::from_utf8(self.trimmed()).ok()?;
        u64::from_str_radix(text, 16).ok()
    }
}

/// The pairs of an input, read on demand, with one pair of look-ahead.
pub(crate) struct Pairs<'a> {
    data: &'a [u8],
    pos: usize,
    line: u64,
    peeked: Option<Pair<'a>>,
}

impl<'a> Pairs<'a> {
    pub(crate) fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            line: 0,
            peeked: None,
        }
    }

    /// The last line read.
    pub(crate) fn line(&self) -> u64 {
        self.peeked.map_or(self.line, |p| p.line)
    }

    fn read_line(&mut self) -> Option<&'a [u8]> {
        if self.pos >= self.data.len() {
            return None;
        }
        let rest = &self.data[self.pos..];
        let end = rest.iter().position(|b| *b == b'\n').unwrap_or(rest.len());
        self.pos += (end + 1).min(rest.len());
        self.line += 1;
        let line = &rest[..end];
        Some(line.strip_suffix(b"\r").unwrap_or(line))
    }

    fn read_pair(&mut self) -> Result<Option<Pair<'a>>, ReadError> {
        let Some(code_line) = self.read_line() else {
            return Ok(None);
        };
        let code_at = self.line;
        let code_text = code_line.trim_ascii();
        let valid = !code_text.is_empty()
            && code_text.len() <= 7
            && code_text
                .iter()
                .enumerate()
                .all(|(i, b)| b.is_ascii_digit() || (i == 0 && (*b == b'-' || *b == b'+')));
        let code = std::str::from_utf8(code_text)
            .ok()
            .filter(|_| valid)
            .and_then(|t| t.parse::<i32>().ok())
            .ok_or(ReadError::BadGroupCode { line: code_at })?;
        let Some(value) = self.read_line() else {
            return Err(ReadError::UnexpectedEnd { line: code_at });
        };
        Ok(Some(Pair {
            code,
            value,
            line: self.line,
        }))
    }

    /// The next pair, or `None` at the end of the input.
    pub(crate) fn next(&mut self) -> Result<Option<Pair<'a>>, ReadError> {
        match self.peeked.take() {
            Some(p) => Ok(Some(p)),
            None => self.read_pair(),
        }
    }

    /// The next pair; the end of the input is an error (the file must end with `0 EOF`).
    pub(crate) fn expect(&mut self) -> Result<Pair<'a>, ReadError> {
        let line = self.line();
        self.next()?.ok_or(ReadError::UnexpectedEnd { line })
    }

    /// Looks at the next pair without taking it.
    pub(crate) fn peek(&mut self) -> Result<Option<Pair<'a>>, ReadError> {
        if self.peeked.is_none() {
            self.peeked = self.read_pair()?;
        }
        Ok(self.peeked)
    }

    /// The next pair of the current entity or record: `None` when the next pair starts another (group code 0).
    /// The end of the input here is an error.
    pub(crate) fn field(&mut self) -> Result<Option<Pair<'a>>, ReadError> {
        match self.peek()? {
            None => Err(ReadError::UnexpectedEnd { line: self.line }),
            Some(p) if p.code == 0 => Ok(None),
            Some(_) => self.next(),
        }
    }

    /// Skips the rest of the current entity or record.
    pub(crate) fn skip_fields(&mut self) -> Result<(), ReadError> {
        while self.field()?.is_some() {}
        Ok(())
    }
}
