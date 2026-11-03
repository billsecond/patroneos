//! Markdown: where each part of a document is, for the formatted view
//! (formatted.rs), and the text changes the formatting commands make. The
//! text itself is never changed by showing it formatted: the view styles
//! it and hides the markup, so a file opened and saved comes back the same.
//!
//! Positions here are in characters from the start of the text, as GTK's
//! text buffers count them.

use std::ops::Range;

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag};

/// How a part of the text looks in the formatted view.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Style {
    Heading(u8),
    Strong,
    Emphasis,
    Strike,
    Code,
    CodeBlock,
    Link,
    Quote,
    ListMarker,
    Table,
    TableHead,
    /// Markup left showing, faintly: code fences, table pipes, the markup
    /// on the line being edited.
    Mark,
    /// A done task's text.
    Done,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub range: Range<usize>,
    pub style: Style,
}

/// Markup hidden in the formatted view, except on the line being edited.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Marker {
    pub range: Range<usize>,
    pub line: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link {
    /// The link's text, which a Ctrl+click opens.
    pub range: Range<usize>,
    pub url: String,
}

/// A task list item's box: `[ ]` or `[x]`, three characters from `start`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Task {
    pub start: usize,
    pub checked: bool,
    pub line: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    pub range: Range<usize>,
    pub line: usize,
    pub url: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    pub range: Range<usize>,
    pub line: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Heading {
    pub level: u8,
    pub start: usize,
    pub text: String,
}

/// Where everything is in a document.
#[derive(Clone, Debug, Default)]
pub struct Parsed {
    pub spans: Vec<Span>,
    pub markers: Vec<Marker>,
    /// What Clear formatting takes out: every marker, and list markers.
    pub removable: Vec<Range<usize>>,
    pub links: Vec<Link>,
    pub tasks: Vec<Task>,
    pub images: Vec<Image>,
    pub rules: Vec<Rule>,
    pub headings: Vec<Heading>,
}

/// File names Notepad treats as Markdown.
pub fn is_markdown_name(name: &str) -> bool {
    let name = name.to_lowercase();
    [".md", ".markdown", ".mdown", ".mkd", ".mkdn", ".mdwn"]
        .iter()
        .any(|extension| name.ends_with(extension))
}

fn options() -> Options {
    Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS
}

/// Byte positions to characters, and characters to lines.
struct Offsets {
    /// Each character's byte position.
    chars: Vec<usize>,
    /// Each line's first character.
    lines: Vec<usize>,
}

impl Offsets {
    fn new(text: &str) -> Offsets {
        let mut chars = Vec::with_capacity(text.len());
        let mut lines = vec![0];
        for (index, (byte, c)) in text.char_indices().enumerate() {
            chars.push(byte);
            if c == '\n' {
                lines.push(index + 1);
            }
        }
        Offsets { chars, lines }
    }

    fn char(&self, byte: usize) -> usize {
        self.chars.partition_point(|&start| start < byte)
    }

    fn range(&self, bytes: Range<usize>) -> Range<usize> {
        self.char(bytes.start)..self.char(bytes.end)
    }

    /// The line a byte is on.
    fn line(&self, byte: usize) -> usize {
        let c = self.char(byte);
        self.lines
            .partition_point(|&start| start <= c)
            .saturating_sub(1)
    }
}

/// A tag the parser has opened, and the parts inside it so far.
struct Frame {
    kind: Kind,
    range: Range<usize>,
    /// Where its first part starts and its last ends (bytes).
    first: Option<usize>,
    last: usize,
    text: String,
}

enum Kind {
    Wrapper(Style),
    Link(String),
    Image(String),
    Heading(u8),
    Item,
    Quote,
    CodeBlock(bool),
    Table,
    TableHead,
    Other,
}

/// The byte ranges of each line's start in `range`, as (line start, line end
/// without its newline).
fn lines_in(text: &str, range: Range<usize>) -> Vec<Range<usize>> {
    let mut lines = Vec::new();
    let mut start = range.start;
    // From the start of the line the range starts on
    while start > 0 && text.as_bytes()[start - 1] != b'\n' {
        start -= 1;
    }
    while start < range.end {
        let end = text[start..range.end]
            .find('\n')
            .map(|n| start + n)
            .unwrap_or(range.end);
        lines.push(start..end);
        start = end + 1;
    }
    lines
}

impl Parsed {
    fn span(&mut self, offsets: &Offsets, bytes: Range<usize>, style: Style) {
        if bytes.start < bytes.end {
            self.spans.push(Span {
                range: offsets.range(bytes),
                style,
            });
        }
    }

    /// Markup that's hidden in the formatted view, and that Clear formatting
    /// takes out.
    fn marker(&mut self, offsets: &Offsets, bytes: Range<usize>) {
        if bytes.start < bytes.end {
            let range = offsets.range(bytes.clone());
            self.markers.push(Marker {
                range: range.clone(),
                line: offsets.line(bytes.start),
            });
            self.removable.push(range);
        }
    }
}

/// Finds every part of a Markdown document.
pub fn parse(text: &str) -> Parsed {
    let offsets = Offsets::new(text);
    let mut out = Parsed::default();
    let mut stack: Vec<Frame> = Vec::new();
    for (event, range) in Parser::new_ext(text, options()).into_offset_iter() {
        // Everything but an end is a part of the tag around it
        if !matches!(event, Event::End(_))
            && let Some(top) = stack.last_mut()
        {
            top.first.get_or_insert(range.start);
            top.last = top.last.max(range.end);
        }
        match event {
            Event::Start(tag) => {
                let kind = match tag {
                    Tag::Strong => Kind::Wrapper(Style::Strong),
                    Tag::Emphasis => Kind::Wrapper(Style::Emphasis),
                    Tag::Strikethrough => Kind::Wrapper(Style::Strike),
                    Tag::Link { dest_url, .. } => Kind::Link(dest_url.to_string()),
                    Tag::Image { dest_url, .. } => Kind::Image(dest_url.to_string()),
                    Tag::Heading { level, .. } => Kind::Heading(level as u8),
                    Tag::Item => Kind::Item,
                    Tag::BlockQuote(_) => Kind::Quote,
                    Tag::CodeBlock(kind) => {
                        Kind::CodeBlock(matches!(kind, CodeBlockKind::Fenced(_)))
                    }
                    Tag::Table(_) => Kind::Table,
                    Tag::TableHead => Kind::TableHead,
                    _ => Kind::Other,
                };
                stack.push(Frame {
                    kind,
                    range: range.clone(),
                    first: None,
                    last: range.start,
                    text: String::new(),
                });
            }
            Event::End(_) => {
                if let Some(frame) = stack.pop() {
                    // A quote in a quote hides its own ">", not the outer one's
                    let depth = stack
                        .iter()
                        .filter(|frame| matches!(frame.kind, Kind::Quote))
                        .count()
                        + 1;
                    finish(text, &offsets, &mut out, frame, depth);
                }
            }
            Event::Text(words) => {
                if let Some(frame) = stack
                    .iter_mut()
                    .rev()
                    .find(|frame| matches!(frame.kind, Kind::Heading(_)))
                {
                    frame.text.push_str(&words);
                }
            }
            Event::Code(code) => {
                if let Some(frame) = stack
                    .iter_mut()
                    .rev()
                    .find(|frame| matches!(frame.kind, Kind::Heading(_)))
                {
                    frame.text.push_str(&code);
                }
                let ticks = text[range.clone()]
                    .bytes()
                    .take_while(|&b| b == b'`')
                    .count();
                if ticks > 0 && range.len() > 2 * ticks {
                    out.marker(&offsets, range.start..range.start + ticks);
                    out.marker(&offsets, range.end - ticks..range.end);
                    out.span(
                        &offsets,
                        range.start + ticks..range.end - ticks,
                        Style::Code,
                    );
                }
            }
            Event::TaskListMarker(checked) => {
                let start = offsets.char(range.start);
                let space = usize::from(text.as_bytes().get(range.end) == Some(&b' '));
                out.removable.push(start..offsets.char(range.end + space));
                out.tasks.push(Task {
                    start,
                    checked,
                    line: offsets.line(range.start),
                });
                if checked {
                    // The rest of the line, struck through
                    let end = text[range.end..]
                        .find('\n')
                        .map(|n| range.end + n)
                        .unwrap_or(text.len());
                    out.span(&offsets, range.end..end, Style::Done);
                }
            }
            Event::Rule => {
                let end = text[range.clone()].trim_end().len() + range.start;
                out.rules.push(Rule {
                    range: offsets.range(range.start..end),
                    line: offsets.line(range.start),
                });
            }
            _ => {}
        }
    }
    out
}

/// What a tag leaves once it's closed: its look, and its markup.
fn finish(text: &str, offsets: &Offsets, out: &mut Parsed, frame: Frame, depth: usize) {
    let range = frame.range.clone();
    match frame.kind {
        Kind::Wrapper(style) => {
            out.span(offsets, range.clone(), style);
            if let Some(first) = frame.first
                && first > range.start
                && frame.last < range.end
            {
                out.marker(offsets, range.start..first);
                out.marker(offsets, frame.last..range.end);
            }
        }
        Kind::Link(url) => {
            if let Some(first) = frame.first
                && first > range.start
            {
                out.marker(offsets, range.start..first);
                out.marker(offsets, frame.last..range.end);
                out.span(offsets, first..frame.last, Style::Link);
                out.links.push(Link {
                    range: offsets.range(first..frame.last),
                    url,
                });
            }
        }
        Kind::Image(url) => out.images.push(Image {
            range: offsets.range(range.clone()),
            line: offsets.line(range.start),
            url,
        }),
        Kind::Heading(level) => {
            let whole = &text[range.clone()];
            let content_end = range.start + whole.trim_end().len();
            out.span(offsets, range.start..content_end, Style::Heading(level));
            let indent = whole.len() - whole.trim_start_matches(' ').len();
            if whole[indent..].starts_with('#') {
                // "## Title ##": the opening marks and the space after them,
                // and any closing marks
                let marks = whole[indent..].bytes().take_while(|&b| b == b'#').count();
                let spaces = whole[indent + marks..]
                    .bytes()
                    .take_while(|&b| b == b' ' || b == b'\t')
                    .count();
                out.marker(offsets, range.start..range.start + indent + marks + spaces);
                let after = &text[frame.last.min(content_end)..content_end];
                if !after.is_empty() && after.trim().bytes().all(|b| b == b'#') {
                    out.marker(offsets, frame.last..content_end);
                }
            } else if let Some(newline) = whole.trim_end().rfind('\n') {
                // An underlined heading: the underline
                out.marker(offsets, range.start + newline + 1..content_end);
            }
            out.headings.push(Heading {
                level,
                start: offsets.char(range.start),
                text: frame.text.trim().to_string(),
            });
        }
        Kind::Item => {
            let indent =
                text[range.clone()].len() - text[range.clone()].trim_start_matches(' ').len();
            let start = range.start + indent;
            let item = &text[start..range.end];
            let numbered = item.bytes().take_while(u8::is_ascii_digit).count();
            let marker =
                if numbered > 0 && matches!(item.as_bytes().get(numbered), Some(b'.' | b')')) {
                    numbered + 1
                } else if matches!(item.as_bytes().first(), Some(b'-' | b'*' | b'+')) {
                    1
                } else {
                    0
                };
            if marker > 0 {
                out.span(offsets, start..start + marker, Style::ListMarker);
                let space = usize::from(item.as_bytes().get(marker) == Some(&b' '));
                out.removable
                    .push(offsets.range(start..start + marker + space));
            }
        }
        Kind::Quote => {
            out.span(offsets, range.clone(), Style::Quote);
            for line in lines_in(text, range) {
                // The line's ">" for this quote: the first for a quote, the
                // second for a quote in it, and so on
                let bytes = text[line.clone()].as_bytes();
                let mut at = 0;
                let mut found = None;
                for _ in 0..depth {
                    while bytes.get(at) == Some(&b' ') {
                        at += 1;
                    }
                    if bytes.get(at) != Some(&b'>') {
                        found = None;
                        break;
                    }
                    let space = usize::from(bytes.get(at + 1) == Some(&b' '));
                    found = Some(at..at + 1 + space);
                    at += 1 + space;
                }
                if let Some(marker) = found {
                    out.marker(offsets, line.start + marker.start..line.start + marker.end);
                }
            }
        }
        Kind::CodeBlock(fenced) => {
            out.span(offsets, range.clone(), Style::CodeBlock);
            if fenced {
                let lines = lines_in(text, range);
                if let Some(open) = lines.first() {
                    out.span(offsets, open.clone(), Style::Mark);
                }
                if lines.len() > 1
                    && let Some(close) = lines.last()
                {
                    let fence = text[close.clone()].trim_start();
                    if fence.starts_with("```") || fence.starts_with("~~~") {
                        out.span(offsets, close.clone(), Style::Mark);
                    }
                }
            }
        }
        Kind::Table => {
            out.span(offsets, range.clone(), Style::Table);
            for line in lines_in(text, range.clone()) {
                let content = &text[line.clone()];
                // The row under the header: |---|:---:|
                let delimiter = content.contains('-')
                    && content
                        .bytes()
                        .all(|b| matches!(b, b'|' | b'-' | b':' | b' ' | b'\t'));
                if delimiter {
                    out.span(offsets, line.clone(), Style::Mark);
                    continue;
                }
                let mut escaped = false;
                for (index, b) in content.bytes().enumerate() {
                    if b == b'|' && !escaped {
                        out.span(
                            offsets,
                            line.start + index..line.start + index + 1,
                            Style::Mark,
                        );
                    }
                    escaped = b == b'\\' && !escaped;
                }
            }
        }
        Kind::TableHead => out.span(offsets, range, Style::TableHead),
        Kind::Other => {}
    }
}

/// The heading level of a line: 1 for "# Title", 0 for plain text.
pub fn heading_level(line: &str) -> u8 {
    let trimmed = line.trim_start_matches(' ');
    let marks = trimmed.bytes().take_while(|&b| b == b'#').count();
    if (1..=6).contains(&marks)
        && matches!(trimmed.as_bytes().get(marks), None | Some(b' ' | b'\t'))
    {
        marks as u8
    } else {
        0
    }
}

/// `line` as a heading of `level` (0 for plain text).
pub fn with_heading(line: &str, level: u8) -> String {
    let trimmed = line.trim_start_matches(' ');
    let body = if heading_level(line) > 0 {
        trimmed
            .trim_start_matches('#')
            .trim_start_matches([' ', '\t'])
    } else {
        trimmed
    };
    if level == 0 {
        body.to_string()
    } else {
        format!("{} {body}", "#".repeat(level as usize))
    }
}

/// A line's list marker: its indent, the marker with the space after it,
/// and whether it's numbered.
fn list_marker(line: &str) -> Option<(usize, usize, bool)> {
    let indent = line.len() - line.trim_start_matches([' ', '\t']).len();
    let rest = &line[indent..];
    let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
    let (length, numbered) =
        if digits > 0 && matches!(rest.as_bytes().get(digits), Some(b'.' | b')')) {
            (digits + 1, true)
        } else if matches!(rest.as_bytes().first(), Some(b'-' | b'*' | b'+')) {
            (1, false)
        } else {
            return None;
        };
    match rest.as_bytes().get(length) {
        Some(b' ') => Some((indent, length + 1, numbered)),
        None => Some((indent, length, numbered)),
        _ => None,
    }
}

/// Bulleted or numbered list on the lines: off when they're all already
/// that kind of list, otherwise on (numbered from 1). Empty lines stay as
/// they are.
pub fn toggle_list(lines: &[&str], numbered: bool) -> Vec<String> {
    let filled: Vec<&&str> = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .collect();
    let all_already = !filled.is_empty()
        && filled
            .iter()
            .all(|line| matches!(list_marker(line), Some((_, _, kind)) if kind == numbered));
    let mut number = 0;
    lines
        .iter()
        .map(|line| {
            if line.trim().is_empty() {
                return line.to_string();
            }
            let (indent, body) = match list_marker(line) {
                Some((indent, length, _)) => (&line[..indent], &line[indent + length..]),
                None => {
                    let indent = line.len() - line.trim_start_matches([' ', '\t']).len();
                    (&line[..indent], &line[indent..])
                }
            };
            if all_already {
                format!("{indent}{body}")
            } else if numbered {
                number += 1;
                format!("{indent}{number}. {body}")
            } else {
                format!("{indent}- {body}")
            }
        })
        .collect()
}

/// A whole HTML page of the document, for Export as HTML.
pub fn to_html(text: &str, title: &str) -> String {
    let mut body = String::new();
    pulldown_cmark::html::push_html(&mut body, Parser::new_ext(text, options()));
    let title = title
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    format!(
        "<!DOCTYPE html>\n<html>\n<head>\n<meta charset=\"utf-8\">\n\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
         <title>{title}</title>\n<style>\n{STYLE}</style>\n</head>\n<body>\n{body}</body>\n</html>\n"
    )
}

const STYLE: &str = "body { font-family: \"PatroneOS Sans\", system-ui, sans-serif; line-height: 1.55; \
max-width: 46em; margin: 2em auto; padding: 0 1em; color: #1b1b1b; }\n\
code, pre { font-family: \"PatroneOS Mono\", ui-monospace, monospace; }\n\
code { background: #f0f2f5; padding: 0.1em 0.3em; border-radius: 4px; }\n\
pre { background: #f0f2f5; padding: 0.8em 1em; border-radius: 6px; overflow-x: auto; }\n\
pre code { background: none; padding: 0; }\n\
blockquote { margin-left: 0; padding-left: 1em; border-left: 3px solid #288bfa; color: #5f6368; }\n\
table { border-collapse: collapse; }\n\
th, td { border: 1px solid #d9dce0; padding: 0.3em 0.7em; }\n\
img { max-width: 100%; }\n\
a { color: #1a6fd4; }\n";

#[cfg(test)]
mod tests {
    use super::*;

    fn slices<'a>(text: &'a str, ranges: impl Iterator<Item = &'a Range<usize>>) -> Vec<String> {
        ranges
            .map(|range| text.chars().skip(range.start).take(range.len()).collect())
            .collect()
    }

    #[test]
    fn finds_inline_markup() {
        let text =
            "Some **bold**, *italic*, ~~gone~~ and `code`, and [a link](https://example.com).";
        let parsed = parse(text);
        let markers = slices(text, parsed.markers.iter().map(|marker| &marker.range));
        assert_eq!(
            markers,
            [
                "**",
                "**",
                "*",
                "*",
                "~~",
                "~~",
                "`",
                "`",
                "[",
                "](https://example.com)"
            ]
        );
        let styles: Vec<Style> = parsed.spans.iter().map(|span| span.style).collect();
        assert!(styles.contains(&Style::Strong));
        assert!(styles.contains(&Style::Emphasis));
        assert!(styles.contains(&Style::Strike));
        assert!(styles.contains(&Style::Code));
        assert_eq!(parsed.links[0].url, "https://example.com");
        assert_eq!(
            slices(text, [&parsed.links[0].range].into_iter()),
            ["a link"]
        );
    }

    #[test]
    fn finds_blocks() {
        let text = "# Title\n\nText.\n\nSub\n---\n\n> quoted\n> more\n\n- one\n- [ ] todo\n- [x] done\n\n1. first\n\n```rust\nfn main() {}\n```\n\n***\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n![logo](images/logo.png)\n";
        let parsed = parse(text);
        let markers = slices(text, parsed.markers.iter().map(|marker| &marker.range));
        assert!(markers.contains(&"# ".to_string()));
        assert!(markers.contains(&"---".to_string()));
        assert_eq!(markers.iter().filter(|marker| *marker == "> ").count(), 2);
        assert_eq!(
            parsed
                .headings
                .iter()
                .map(|heading| (heading.level, heading.text.as_str()))
                .collect::<Vec<_>>(),
            [(1, "Title"), (2, "Sub")]
        );
        assert_eq!(parsed.tasks.len(), 2);
        assert!(!parsed.tasks[0].checked && parsed.tasks[1].checked);
        let boxes: Vec<String> = parsed
            .tasks
            .iter()
            .map(|task| text.chars().skip(task.start).take(3).collect())
            .collect();
        assert_eq!(boxes, ["[ ]", "[x]"]);
        assert_eq!(parsed.rules.len(), 1);
        assert_eq!(slices(text, [&parsed.rules[0].range].into_iter()), ["***"]);
        assert_eq!(parsed.images[0].url, "images/logo.png");
        let styles: Vec<Style> = parsed.spans.iter().map(|span| span.style).collect();
        for style in [
            Style::Quote,
            Style::ListMarker,
            Style::CodeBlock,
            Style::Table,
            Style::TableHead,
            Style::Done,
        ] {
            assert!(styles.contains(&style), "{style:?}");
        }
    }

    #[test]
    fn hides_each_quotes_own_mark() {
        let text = "> outer
> > inner
";
        let parsed = parse(text);
        let mut markers: Vec<Range<usize>> = parsed
            .markers
            .iter()
            .map(|marker| marker.range.clone())
            .collect();
        markers.sort_by_key(|range| range.start);
        assert_eq!(markers, [0..2, 8..10, 10..12]);
    }

    #[test]
    fn finds_indented_list_markers() {
        let text = "- one
  - two
";
        let parsed = parse(text);
        let markers: Vec<Range<usize>> = parsed
            .spans
            .iter()
            .filter(|span| span.style == Style::ListMarker)
            .map(|span| span.range.clone())
            .collect();
        assert_eq!(markers, [8..9, 0..1]);
    }

    #[test]
    fn counts_characters_not_bytes() {
        let text = "Café — **été**\n";
        let parsed = parse(text);
        let markers = slices(text, parsed.markers.iter().map(|marker| &marker.range));
        assert_eq!(markers, ["**", "**"]);
        assert_eq!(parsed.markers[0].range, 7..9);
    }

    #[test]
    fn leaves_plain_words_alone() {
        // An underscore inside a word isn't emphasis (an asterisk can be)
        let parsed = parse("snake_case_name and 2 * 3 * 4 and a_b\n");
        assert!(parsed.markers.is_empty());
    }

    #[test]
    fn changes_headings() {
        assert_eq!(heading_level("## Two"), 2);
        assert_eq!(heading_level("#hashtag"), 0);
        assert_eq!(heading_level("plain"), 0);
        assert_eq!(with_heading("Title", 1), "# Title");
        assert_eq!(with_heading("### Title", 2), "## Title");
        assert_eq!(with_heading("## Title", 0), "Title");
    }

    #[test]
    fn toggles_lists() {
        assert_eq!(
            toggle_list(&["one", "", "two"], false),
            ["- one", "", "- two"]
        );
        assert_eq!(toggle_list(&["- one", "- two"], false), ["one", "two"]);
        assert_eq!(
            toggle_list(&["- one", "  - two"], true),
            ["1. one", "  2. two"]
        );
        assert_eq!(toggle_list(&["1. one", "2. two"], true), ["one", "two"]);
    }

    #[test]
    fn knows_markdown_names() {
        assert!(is_markdown_name("README.md"));
        assert!(is_markdown_name("notes.Markdown"));
        assert!(!is_markdown_name("notes.txt"));
    }

    #[test]
    fn exports_a_page() {
        let html = to_html("# Hi & bye\n\n**bold**\n", "Notes & <more>");
        assert!(html.contains("<title>Notes &amp; &lt;more&gt;</title>"));
        assert!(html.contains("<h1>Hi &amp; bye</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
    }
}
