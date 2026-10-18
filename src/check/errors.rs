//! The Check section of `rules/standards/errors.md`: a boxed `dyn Error`,
//! a snafu variant that loses its cause or its location, the untyped
//! snafu forms, and `source()` walked outside `error.rs`. Banned crates
//! are in `manifest.rs`.

use super::source::{
    Source, compact, from, has_word, is_ident, matching, skip_blank, slice, tokens, top_level_items,
};
use super::{Rule, finding};

pub fn check(src: &Source, out: &mut Vec<super::Finding>) {
    for line in src.lines() {
        if boxes_dyn_error(line.masked) {
            out.push(finding(
                src,
                line.number,
                Rule::BoxedError,
                line.text.trim(),
            ));
        }
        for what in untyped_snafu(line.masked) {
            out.push(finding(src, line.number, Rule::UntypedSnafu, what));
        }
        if line.masked.contains(".source()")
            && !src.is_error_module()
            && !src.is_test_at(line.offset)
        {
            out.push(finding(
                src,
                line.number,
                Rule::SourceCall,
                ".source() outside error.rs and tests",
            ));
        }
    }
    snafu_use_lists(src, out);
    snafu_items(src, out);
}

/// `Box<dyn ... Error ...>` anywhere in the line, whatever the spacing.
fn boxes_dyn_error(line: &str) -> bool {
    let squeezed = compact(line);
    squeezed.match_indices("Box<dyn").any(|(at, needle)| {
        from(&squeezed, at + needle.len())
            .split('>')
            .next()
            .is_some_and(|inner| inner.contains("Error"))
    })
}

/// The untyped snafu forms on one line, in a fixed order.
fn untyped_snafu(line: &str) -> Vec<&'static str> {
    let mut hits = Vec::new();
    if compact(line).contains("context(false)") {
        hits.push("context(false)");
    }
    if has_word(line, "Whatever") {
        hits.push("Whatever");
    }
    if !tokens(line, "whatever!").is_empty() {
        hits.push("whatever!");
    }
    if has_word(line, "snafu::Report") {
        hits.push("snafu::Report");
    }
    hits
}

/// `Report` inside a `use snafu::{ ... }` list, which may span lines.
fn snafu_use_lists(src: &Source, out: &mut Vec<super::Finding>) {
    let needle = "use snafu::{";
    for (at, _) in src.masked.match_indices(needle) {
        let open = at + needle.len() - 1;
        let Some(close) = matching(&src.masked, open) else {
            continue;
        };
        let mut pos = open + 1;
        for item in slice(&src.masked, open + 1, close).split(',') {
            let name = item.split_whitespace().next().unwrap_or_default();
            if name == "Report" {
                out.push(finding(
                    src,
                    src.line_of(pos + item.len() - item.trim_start().len()),
                    Rule::UntypedSnafu,
                    "snafu::Report",
                ));
            }
            pos += item.len() + 1;
        }
    }
}

/// An item in masked text: its attributes, its name and its brace body.
struct Item {
    attrs: (usize, usize),
    name: String,
    name_at: usize,
    /// The index after the name, where a field's `: Type` begins.
    after: usize,
    body: Option<(usize, usize)>,
}

/// Parse `#[attr] name { ... }` or `#[attr] name: Type` in `start..end`.
fn parse_item(masked: &str, start: usize, end: usize) -> Option<Item> {
    let bytes = masked.as_bytes();
    let mut pos = skip_blank(bytes, start, end);
    let attrs_start = pos;
    while bytes.get(pos) == Some(&b'#') && bytes.get(pos + 1) == Some(&b'[') {
        pos = matching(masked, pos + 1)? + 1;
        pos = skip_blank(bytes, pos, end);
    }
    let attrs_end = pos;
    let (mut name, mut name_at) = ident_at(masked, pos, end);
    pos = name_at + name.len();
    if name == "pub" {
        pos = skip_blank(bytes, pos, end);
        if bytes.get(pos) == Some(&b'(') {
            pos = matching(masked, pos)? + 1;
            pos = skip_blank(bytes, pos, end);
        }
        (name, name_at) = ident_at(masked, pos, end);
        pos = name_at + name.len();
    }
    if name.is_empty() {
        return None;
    }
    let after = pos;
    pos = skip_blank(bytes, pos, end);
    let body = (bytes.get(pos) == Some(&b'{'))
        .then(|| matching(masked, pos).map(|close| (pos, close)))
        .flatten();
    Some(Item {
        attrs: (attrs_start, attrs_end),
        name,
        name_at,
        after,
        body,
    })
}

/// The identifier starting at `pos`, and where it starts.
fn ident_at(masked: &str, pos: usize, end: usize) -> (String, usize) {
    let bytes = masked.as_bytes();
    let mut stop = pos;
    while stop < end && bytes.get(stop).is_some_and(|&b| is_ident(b)) {
        stop += 1;
    }
    (slice(masked, pos, stop).to_owned(), pos)
}

/// Every `#[derive(... Snafu ...)]` enum or struct.
fn snafu_items(src: &Source, out: &mut Vec<super::Finding>) {
    let masked = src.masked.as_str();
    for (at, _) in masked.match_indices("#[derive(") {
        let open = at + "#[derive".len();
        let Some(close) = matching(masked, open) else {
            continue;
        };
        if !has_word(slice(masked, open, close), "Snafu") {
            continue;
        }
        let Some(body_open) = from(masked, close + 1).find('{').map(|i| close + 1 + i) else {
            continue;
        };
        let header = slice(masked, close + 1, body_open);
        if header.contains(';') {
            continue;
        }
        let Some(body_close) = matching(masked, body_open) else {
            continue;
        };
        if has_word(header, "enum") {
            for (start, end) in top_level_items(masked, body_open + 1, body_close) {
                if let Some(variant) = parse_item(masked, start, end) {
                    inspect(src, &variant, out);
                }
            }
        } else if let Some(keyword) = tokens(header, "struct").first() {
            let keyword = close + 1 + keyword;
            let (name, name_at) = ident_at(
                masked,
                skip_blank(masked.as_bytes(), keyword + "struct".len(), body_open),
                body_open,
            );
            let item = Item {
                attrs: (attrs_before(masked, at), keyword),
                name,
                name_at,
                after: name_at,
                body: Some((body_open, body_close)),
            };
            inspect(src, &item, out);
        }
    }
}

/// The start of the run of attributes ending just before `at`, so a
/// struct's `#[snafu(display(..))]` is seen wherever it sits.
fn attrs_before(masked: &str, at: usize) -> usize {
    let bytes = masked.as_bytes();
    let mut start = at;
    loop {
        let mut pos = start;
        while pos > 0 && bytes[pos - 1].is_ascii_whitespace() {
            pos -= 1;
        }
        if pos == 0 || bytes[pos - 1] != b']' {
            return start;
        }
        let mut depth = 0usize;
        let mut open = None;
        for i in (0..pos).rev() {
            match bytes[i] {
                b']' => depth += 1,
                b'[' => {
                    depth -= 1;
                    if depth == 0 {
                        open = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }
        match open {
            Some(open) if open > 0 && bytes[open - 1] == b'#' => start = open - 1,
            _ => return start,
        }
    }
}

/// A variant with a `source` field must display `{source}` and carry
/// `#[snafu(implicit)] location: Loc`.
fn inspect(src: &Source, item: &Item, out: &mut Vec<super::Finding>) {
    let Some((open, close)) = item.body else {
        return;
    };
    let masked = src.masked.as_str();
    let mut has_source = false;
    let mut has_location = false;
    for (start, end) in top_level_items(masked, open + 1, close) {
        let Some(field) = parse_item(masked, start, end) else {
            continue;
        };
        let attrs = compact(slice(masked, field.attrs.0, field.attrs.1));
        let ty = slice(masked, field.after, end)
            .trim()
            .trim_start_matches(':')
            .trim();
        let is_source = !attrs.contains("source(false)")
            && (field.name == "source" || attrs.contains("snafu(source"));
        has_source |= is_source;
        if field.name == "location"
            && attrs.contains("implicit")
            && (ty == "Loc" || ty.ends_with("::Loc"))
        {
            has_location = true;
        }
    }
    if !has_source {
        return;
    }
    let line = src.line_of(item.name_at);
    let name = &item.name;
    match display_string(src, item.attrs) {
        None => out.push(finding(
            src,
            line,
            Rule::VariantDisplay,
            format!("{name} has a source but no display string"),
        )),
        Some(display) if !display.contains("{source}") => out.push(finding(
            src,
            line,
            Rule::VariantDisplay,
            format!("{name} has a source but its display lacks {{source}}"),
        )),
        Some(_) => {}
    }
    if !has_location {
        out.push(finding(
            src,
            line,
            Rule::VariantLocation,
            format!("{name} has a source but no #[snafu(implicit)] location: Loc"),
        ));
    }
}

/// The first string literal after `display` in the attribute range, read
/// from the unmasked text.
fn display_string(src: &Source, attrs: (usize, usize)) -> Option<&str> {
    let masked = slice(&src.masked, attrs.0, attrs.1);
    let display = tokens(masked, "display").first().copied()?;
    let open = attrs.0 + display + from(masked, display).find('"')?;
    let close = open + 1 + from(&src.masked, open + 1).find('"')?;
    Some(slice(&src.text, open + 1, close))
}
