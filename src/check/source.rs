//! A Rust file as the rules read it: the text, and a masked copy of the
//! same length in which every byte of a comment or a string literal is
//! `#`. A pattern in a doc example or a message is then never a hit, and
//! a byte offset names the same place in both copies.

use std::path::PathBuf;

pub struct Source {
    /// Relative to the root of the check.
    pub path: PathBuf,
    /// The file as read.
    pub text: String,
    /// `text` with comments and string literals masked.
    pub masked: String,
    line_starts: Vec<usize>,
    cfg_test: Option<usize>,
    under_tests: bool,
}

/// One line of a source, in both copies.
pub struct Line<'a> {
    pub number: usize,
    pub offset: usize,
    pub masked: &'a str,
    pub text: &'a str,
}

impl Source {
    pub fn new(path: PathBuf, text: String) -> Self {
        let masked = mask(&text);
        let line_starts = std::iter::once(0)
            .chain(text.match_indices('\n').map(|(at, _)| at + 1))
            .collect();
        let cfg_test = masked.find("#[cfg(test)]");
        let under_tests = path.components().any(|c| c.as_os_str() == "tests");
        Source {
            path,
            text,
            masked,
            line_starts,
            cfg_test,
            under_tests,
        }
    }

    /// The 1-based line holding byte `offset`.
    pub fn line_of(&self, offset: usize) -> usize {
        self.line_starts.partition_point(|&start| start <= offset)
    }

    /// Whether byte `offset` is test code: the file is under `tests/`, or
    /// the offset follows the first `#[cfg(test)]`.
    pub fn is_test_at(&self, offset: usize) -> bool {
        self.under_tests || self.cfg_test.is_some_and(|at| offset >= at)
    }

    /// `main.rs`, or anything under `src/bin`.
    pub fn is_main(&self) -> bool {
        self.path.file_name().is_some_and(|name| name == "main.rs")
            || self
                .path
                .parent()
                .is_some_and(|dir| dir.ends_with("src/bin"))
    }

    /// `error.rs`, or anything under a directory named `error`.
    pub fn is_error_module(&self) -> bool {
        self.path.file_name().is_some_and(|name| name == "error.rs")
            || self.path.components().any(|c| c.as_os_str() == "error")
    }

    pub fn lines(&self) -> impl Iterator<Item = Line<'_>> {
        self.line_starts.iter().enumerate().map(move |(i, &start)| {
            let end = self
                .line_starts
                .get(i + 1)
                .map_or(self.text.len(), |&next| next - 1);
            Line {
                number: i + 1,
                offset: start,
                masked: slice(&self.masked, start, end),
                text: slice(&self.text, start, end),
            }
        })
    }

    /// Every offset in the masked text where `needle` starts a token.
    pub fn tokens(&self, needle: &str) -> Vec<usize> {
        tokens(&self.masked, needle)
    }
}

/// `s[start..end]`, or empty when the range is not valid.
pub fn slice(s: &str, start: usize, end: usize) -> &str {
    s.get(start..end).unwrap_or("")
}

/// `s[start..]`, or empty when the offset is not valid.
pub fn from(s: &str, start: usize) -> &str {
    s.get(start..).unwrap_or("")
}

/// `s` without whitespace.
pub fn compact(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

pub fn is_ident(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// Every offset where `needle` occurs and the byte before is not part of
/// an identifier, so `catch_unwind(` is found and `my_catch_unwind(` is
/// not.
pub fn tokens(haystack: &str, needle: &str) -> Vec<usize> {
    let bytes = haystack.as_bytes();
    haystack
        .match_indices(needle)
        .map(|(at, _)| at)
        .filter(|&at| at == 0 || !is_ident(bytes[at - 1]))
        .collect()
}

/// Whether `word` occurs with no identifier byte on either side.
pub fn has_word(haystack: &str, word: &str) -> bool {
    let bytes = haystack.as_bytes();
    tokens(haystack, word)
        .iter()
        .any(|&at| bytes.get(at + word.len()).is_none_or(|&b| !is_ident(b)))
}

/// The index of the bracket closing the one at `open`, in masked text.
pub fn matching(haystack: &str, open: usize) -> Option<usize> {
    let bytes = haystack.as_bytes();
    let (lhs, rhs) = match bytes.get(open)? {
        b'(' => (b'(', b')'),
        b'[' => (b'[', b']'),
        b'{' => (b'{', b'}'),
        _ => return None,
    };
    let mut depth = 0usize;
    for (i, &b) in bytes.iter().enumerate().skip(open) {
        if b == lhs {
            depth += 1;
        } else if b == rhs {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}

/// The first index at or after `pos` that is not whitespace and not a
/// masked comment, stopping before `end`. A masked comment is a run of
/// `#` that does not open an attribute.
pub fn skip_blank(bytes: &[u8], mut pos: usize, end: usize) -> usize {
    while pos < end {
        match bytes[pos] {
            b if b.is_ascii_whitespace() => pos += 1,
            b'#' if bytes.get(pos + 1) != Some(&b'[')
                && bytes.get(pos + 1..pos + 3) != Some(b"![") =>
            {
                pos += 1;
            }
            _ => break,
        }
    }
    pos
}

/// The ranges between top-level commas in `start..end` of masked text,
/// where top level means outside every bracket.
pub fn top_level_items(haystack: &str, start: usize, end: usize) -> Vec<(usize, usize)> {
    let bytes = haystack.as_bytes();
    let mut items = Vec::new();
    let mut depth = 0usize;
    let mut item_start = start;
    for i in start..end.min(bytes.len()) {
        match bytes[i] {
            b'(' | b'[' | b'{' | b'<' => depth += 1,
            b'>' if i > 0 && bytes[i - 1] == b'-' => {}
            b')' | b']' | b'}' | b'>' => depth = depth.saturating_sub(1),
            b',' if depth == 0 => {
                items.push((item_start, i));
                item_start = i + 1;
            }
            _ => {}
        }
    }
    items.push((item_start, end));
    items
}

fn mask(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = bytes.to_vec();
    let mut i = 0;
    while i < bytes.len() {
        let prev_ident = i > 0 && is_ident(bytes[i - 1]);
        let next = bytes.get(i + 1).copied();
        match bytes[i] {
            b'/' if next == Some(b'/') => {
                let end = line_end(bytes, i);
                cover(&mut out, i, end);
                i = end;
            }
            b'/' if next == Some(b'*') => {
                let end = block_comment_end(bytes, i + 2);
                cover(&mut out, i, end);
                i = end;
            }
            b'"' => {
                let end = string_end(bytes, i + 1);
                cover(&mut out, i + 1, end);
                i = end + 1;
            }
            b'r' | b'b' | b'c' if !prev_ident => match raw_string(bytes, i) {
                Some((start, end, after)) => {
                    cover(&mut out, start, end);
                    i = after;
                }
                None => i += 1,
            },
            b'\'' => match char_literal_end(bytes, i) {
                Some(end) => {
                    cover(&mut out, i + 1, end);
                    i = end + 1;
                }
                None => i += 1,
            },
            _ => i += 1,
        }
    }
    // Only ASCII bytes were replaced, by ASCII, so this cannot fail; the
    // fallback keeps the offsets right regardless.
    String::from_utf8(out).unwrap_or_else(|_| text.to_owned())
}

/// Replace every ASCII byte in `start..end` with `#`, keeping line breaks
/// and multi-byte characters so offsets and line numbers hold.
fn cover(out: &mut [u8], start: usize, end: usize) {
    if let Some(range) = out.get_mut(start..end) {
        for b in range {
            if b.is_ascii() && *b != b'\n' && *b != b'\r' {
                *b = b'#';
            }
        }
    }
}

fn line_end(bytes: &[u8], from: usize) -> usize {
    bytes
        .iter()
        .skip(from)
        .position(|&b| b == b'\n')
        .map_or(bytes.len(), |n| from + n)
}

/// The index after the `*/` closing a block comment whose `/*` ended just
/// before `from`; nested comments are honoured as Rust does.
fn block_comment_end(bytes: &[u8], from: usize) -> usize {
    let mut depth = 1usize;
    let mut i = from;
    while i + 1 < bytes.len() {
        if bytes[i] == b'/' && bytes[i + 1] == b'*' {
            depth += 1;
            i += 2;
        } else if bytes[i] == b'*' && bytes[i + 1] == b'/' {
            depth -= 1;
            i += 2;
            if depth == 0 {
                return i;
            }
        } else {
            i += 1;
        }
    }
    bytes.len()
}

/// The index of the quote closing a string whose opening quote ended just
/// before `from`.
fn string_end(bytes: &[u8], from: usize) -> usize {
    let mut i = from;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b'"' => return i,
            _ => i += 1,
        }
    }
    bytes.len()
}

/// A raw string starting at `at` (`r"`, `r#"`, `br"`, `cr"`): the content
/// range and the index after the closing delimiter.
fn raw_string(bytes: &[u8], at: usize) -> Option<(usize, usize, usize)> {
    let mut i = at;
    if matches!(bytes.get(i), Some(b'b' | b'c')) {
        i += 1;
    }
    if bytes.get(i) != Some(&b'r') {
        return None;
    }
    i += 1;
    let hashes = bytes.iter().skip(i).take_while(|&&b| b == b'#').count();
    i += hashes;
    if bytes.get(i) != Some(&b'"') {
        return None;
    }
    let start = i + 1;
    let mut k = start;
    while k < bytes.len() {
        let closes = bytes[k] == b'"'
            && bytes
                .get(k + 1..k + 1 + hashes)
                .is_some_and(|tail| tail.iter().all(|&b| b == b'#'));
        if closes {
            return Some((start, k, k + 1 + hashes));
        }
        k += 1;
    }
    Some((start, bytes.len(), bytes.len()))
}

/// The closing quote of a char literal opening at `at`, or `None` when the
/// quote starts a lifetime or a label.
fn char_literal_end(bytes: &[u8], at: usize) -> Option<usize> {
    match *bytes.get(at + 1)? {
        b'\\' => bytes
            .iter()
            .enumerate()
            .skip(at + 3)
            .take(12)
            .find(|&(_, &b)| b == b'\'')
            .map(|(i, _)| i),
        first => {
            let width = match first {
                0x00..=0x7F => 1,
                0xC0..=0xDF => 2,
                0xE0..=0xEF => 3,
                _ => 4,
            };
            (bytes.get(at + 1 + width) == Some(&b'\'')).then_some(at + 1 + width)
        }
    }
}
