//! A text file's bytes and the text Notepad shows: which encoding and line
//! endings a file has, and writing the text back the same way. The text
//! itself always has plain `\n` line endings; a file's own come back when
//! it's saved.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Encoding {
    Utf8,
    Utf8Bom,
    Utf16Le,
    Utf16Be,
    Cp1252,
}

impl Encoding {
    pub const ALL: [Encoding; 5] = [
        Encoding::Utf8,
        Encoding::Utf8Bom,
        Encoding::Utf16Le,
        Encoding::Utf16Be,
        Encoding::Cp1252,
    ];

    /// The name in the status bar and its menu.
    pub fn label(self) -> &'static str {
        match self {
            Encoding::Utf8 => "UTF-8",
            Encoding::Utf8Bom => "UTF-8 with BOM",
            Encoding::Utf16Le => "UTF-16 LE",
            Encoding::Utf16Be => "UTF-16 BE",
            Encoding::Cp1252 => "Western (CP1252)",
        }
    }

    /// The name in actions and the saved session.
    pub fn key(self) -> &'static str {
        match self {
            Encoding::Utf8 => "utf-8",
            Encoding::Utf8Bom => "utf-8-bom",
            Encoding::Utf16Le => "utf-16le",
            Encoding::Utf16Be => "utf-16be",
            Encoding::Cp1252 => "cp1252",
        }
    }

    pub fn from_key(key: &str) -> Option<Encoding> {
        Encoding::ALL
            .into_iter()
            .find(|encoding| encoding.key() == key)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LineEnding {
    Lf,
    CrLf,
    Cr,
}

impl LineEnding {
    pub const ALL: [LineEnding; 3] = [LineEnding::Lf, LineEnding::CrLf, LineEnding::Cr];

    /// The name in the status bar and its menu.
    pub fn label(self) -> &'static str {
        match self {
            LineEnding::Lf => "LF",
            LineEnding::CrLf => "CRLF",
            LineEnding::Cr => "CR",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            LineEnding::Lf => "lf",
            LineEnding::CrLf => "crlf",
            LineEnding::Cr => "cr",
        }
    }

    pub fn from_key(key: &str) -> Option<LineEnding> {
        LineEnding::ALL
            .into_iter()
            .find(|ending| ending.key() == key)
    }

    fn sequence(self) -> &'static str {
        match self {
            LineEnding::Lf => "\n",
            LineEnding::CrLf => "\r\n",
            LineEnding::Cr => "\r",
        }
    }
}

/// A file read for Notepad.
#[derive(Debug, PartialEq, Eq)]
pub struct Decoded {
    pub text: String,
    pub encoding: Encoding,
    pub line_ending: LineEnding,
}

/// CP1252's characters for its bytes 0x80 to 0x9F. The five bytes it leaves
/// undefined stand for the control characters of the same number, so every
/// byte reads as something and writes back as itself.
const CP1252_HIGH: [char; 32] = [
    '\u{20AC}', '\u{0081}', '\u{201A}', '\u{0192}', '\u{201E}', '\u{2026}', '\u{2020}', '\u{2021}',
    '\u{02C6}', '\u{2030}', '\u{0160}', '\u{2039}', '\u{0152}', '\u{008D}', '\u{017D}', '\u{008F}',
    '\u{0090}', '\u{2018}', '\u{2019}', '\u{201C}', '\u{201D}', '\u{2022}', '\u{2013}', '\u{2014}',
    '\u{02DC}', '\u{2122}', '\u{0161}', '\u{203A}', '\u{0153}', '\u{009D}', '\u{017E}', '\u{0178}',
];

fn cp1252_char(byte: u8) -> char {
    match byte {
        0x80..=0x9F => CP1252_HIGH[(byte - 0x80) as usize],
        _ => byte as char,
    }
}

fn cp1252_byte(c: char) -> Option<u8> {
    match c as u32 {
        0..=0x7F | 0xA0..=0xFF => Some(c as u8),
        _ => CP1252_HIGH
            .iter()
            .position(|&high| high == c)
            .map(|index| 0x80 + index as u8),
    }
}

fn utf16(bytes: &[u8], little_endian: bool) -> String {
    let units = bytes.chunks(2).map(|pair| match (pair, little_endian) {
        ([low, high], true) => u16::from_le_bytes([*low, *high]),
        ([high, low], false) => u16::from_be_bytes([*high, *low]),
        // A stray last byte
        _ => 0xFFFD,
    });
    char::decode_utf16(units)
        .map(|unit| unit.unwrap_or(char::REPLACEMENT_CHARACTER))
        .collect()
}

/// The line ending used most in `text` (LF when it has none).
fn line_ending_of(text: &str) -> LineEnding {
    let bytes = text.as_bytes();
    let (mut lf, mut crlf, mut cr) = (0, 0, 0);
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'\r' if bytes.get(index + 1) == Some(&b'\n') => {
                crlf += 1;
                index += 1;
            }
            b'\r' => cr += 1,
            b'\n' => lf += 1,
            _ => {}
        }
        index += 1;
    }
    if crlf > lf && crlf >= cr {
        LineEnding::CrLf
    } else if cr > lf && cr > crlf {
        LineEnding::Cr
    } else {
        LineEnding::Lf
    }
}

/// Reads a file's bytes: a byte order mark says UTF-8 or UTF-16; otherwise
/// it's UTF-8 if it can be, and CP1252, which every byte can be, if not.
pub fn decode(bytes: &[u8]) -> Decoded {
    let (text, encoding) = if let Some(rest) = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
        (
            String::from_utf8_lossy(rest).into_owned(),
            Encoding::Utf8Bom,
        )
    } else if let Some(rest) = bytes.strip_prefix(&[0xFF, 0xFE]) {
        (utf16(rest, true), Encoding::Utf16Le)
    } else if let Some(rest) = bytes.strip_prefix(&[0xFE, 0xFF]) {
        (utf16(rest, false), Encoding::Utf16Be)
    } else if let Ok(text) = std::str::from_utf8(bytes) {
        (text.to_owned(), Encoding::Utf8)
    } else {
        (
            bytes.iter().map(|&byte| cp1252_char(byte)).collect(),
            Encoding::Cp1252,
        )
    };
    let line_ending = line_ending_of(&text);
    // A text view can't hold a NUL character
    let text = text
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\0', "\u{FFFD}");
    Decoded {
        text,
        encoding,
        line_ending,
    }
}

/// How many of `text`'s characters `encoding` can't write (each becomes a
/// question mark).
pub fn unwritable(text: &str, encoding: Encoding) -> usize {
    match encoding {
        Encoding::Cp1252 => text.chars().filter(|&c| cp1252_byte(c).is_none()).count(),
        _ => 0,
    }
}

/// The bytes to save: `text` with `line_ending`s, in `encoding`.
pub fn encode(text: &str, encoding: Encoding, line_ending: LineEnding) -> Vec<u8> {
    let text = if line_ending == LineEnding::Lf {
        std::borrow::Cow::Borrowed(text)
    } else {
        std::borrow::Cow::Owned(text.replace('\n', line_ending.sequence()))
    };
    match encoding {
        Encoding::Utf8 => text.as_bytes().to_vec(),
        Encoding::Utf8Bom => [&[0xEF, 0xBB, 0xBF][..], text.as_bytes()].concat(),
        Encoding::Utf16Le => [0xFF, 0xFE]
            .into_iter()
            .chain(text.encode_utf16().flat_map(u16::to_le_bytes))
            .collect(),
        Encoding::Utf16Be => [0xFE, 0xFF]
            .into_iter()
            .chain(text.encode_utf16().flat_map(u16::to_be_bytes))
            .collect(),
        Encoding::Cp1252 => text
            .chars()
            .map(|c| cp1252_byte(c).unwrap_or(b'?'))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_utf8_and_its_line_endings() {
        let decoded = decode(b"one\r\ntwo\r\nthree");
        assert_eq!(decoded.text, "one\ntwo\nthree");
        assert_eq!(decoded.encoding, Encoding::Utf8);
        assert_eq!(decoded.line_ending, LineEnding::CrLf);
        assert_eq!(decode(b"a\nb").line_ending, LineEnding::Lf);
        assert_eq!(decode(b"a\rb\rc").line_ending, LineEnding::Cr);
        assert_eq!(decode(b"no line ending").line_ending, LineEnding::Lf);
        assert_eq!(decode("caf\u{e9}".as_bytes()).text, "caf\u{e9}");
    }

    #[test]
    fn reads_byte_order_marks() {
        assert_eq!(decode(b"\xEF\xBB\xBFhi").encoding, Encoding::Utf8Bom);
        assert_eq!(decode(b"\xEF\xBB\xBFhi").text, "hi");
        let le = decode(b"\xFF\xFEh\0i\0\r\0\n\0");
        assert_eq!((le.text.as_str(), le.encoding), ("hi\n", Encoding::Utf16Le));
        assert_eq!(le.line_ending, LineEnding::CrLf);
        let be = decode(b"\xFE\xFF\0h\0i");
        assert_eq!((be.text.as_str(), be.encoding), ("hi", Encoding::Utf16Be));
    }

    #[test]
    fn reads_other_bytes_as_cp1252() {
        let decoded = decode(b"\x93quoted\x94 caf\xE9 \x80");
        assert_eq!(decoded.encoding, Encoding::Cp1252);
        assert_eq!(decoded.text, "\u{201C}quoted\u{201D} caf\u{e9} \u{20AC}");
        assert_eq!(
            encode(&decoded.text, Encoding::Cp1252, LineEnding::Lf),
            b"\x93quoted\x94 caf\xE9 \x80"
        );
    }

    #[test]
    fn writes_what_it_read() {
        for encoding in Encoding::ALL {
            for line_ending in LineEnding::ALL {
                let text = "first line\nsecond, caf\u{e9}\n\nlast";
                let bytes = encode(text, encoding, line_ending);
                let decoded = decode(&bytes);
                assert_eq!(decoded.text, text, "{encoding:?} {line_ending:?}");
                assert_eq!(decoded.line_ending, line_ending, "{encoding:?}");
                // Plain ASCII in UTF-8 and CP1252 is the same bytes, and reads
                // as UTF-8; with é it's only valid in one of them.
                assert_eq!(decoded.encoding, encoding);
            }
        }
    }

    #[test]
    fn counts_what_cp1252_cant_write() {
        assert_eq!(unwritable("caf\u{e9} \u{20AC}", Encoding::Cp1252), 0);
        assert_eq!(
            unwritable("\u{65e5}\u{672c} \u{1F600}", Encoding::Cp1252),
            3
        );
        assert_eq!(unwritable("\u{65e5}\u{672c}", Encoding::Utf8), 0);
        assert_eq!(
            encode("a\u{65e5}b", Encoding::Cp1252, LineEnding::Lf),
            b"a?b"
        );
    }

    #[test]
    fn keys_name_every_choice() {
        for encoding in Encoding::ALL {
            assert_eq!(Encoding::from_key(encoding.key()), Some(encoding));
        }
        for ending in LineEnding::ALL {
            assert_eq!(LineEnding::from_key(ending.key()), Some(ending));
        }
    }
}
