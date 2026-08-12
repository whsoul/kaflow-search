//! Splitting text into the terms a search works with.
//!
//! Indexing and searching both pass through this, which is the point: a term that was
//! never produced when indexing can never be found when searching, so the two must split
//! text the same way.
//!
//! What this crate holds is the contract and a dictionary-free implementation of it, with
//! no dependencies — enough for anyone to use it, while a heavier implementation can be
//! substituted where one is available.
//!
//! ⚠️ **Which implementation is used is fixed when the build is made, not at run time.**
//! An index built by one tokenizer does not agree with searches split by another; changing
//! it means re-indexing, so it cannot be a setting.

/// What every tokenizer must do.
///
/// Indexing and searching must call the same one — that is what keeps them agreeing.
pub trait Tokenizer {
    /// Splits text into terms. Empty ones are not returned.
    fn tokenize(&self, text: &str) -> Vec<String>;
}

/// Punctuation that ends a word. Whitespace is handled separately.
///
/// `@ - _ /` are deliberately absent: they hold together addresses, paths and identifiers,
/// and splitting on them would make those unsearchable as written. `.` does end sentences
/// and so is included, at some cost to version-like strings.
pub const TOKENIZE_DELIMITERS: &[char] = &[
    '?', '!', ',', '.', ';', ':', '"', '\'', '(', ')', '[', ']', '{', '}',
];

/// Splits on whitespace and punctuation, with a fallback for scripts that use neither.
///
/// Languages written with spaces are served well by this. Chinese and Japanese are not —
/// a whole sentence would become one term — so their characters are split individually.
/// That is coarse rather than correct, but it needs no dictionary and always gives the
/// same answer, which is what a fallback has to do.
#[derive(Debug, Default, Clone, Copy)]
pub struct WhitespaceTokenizer;

impl Tokenizer for WhitespaceTokenizer {
    fn tokenize(&self, text: &str) -> Vec<String> {
        whitespace_tokenize(text)
    }
}

/// Splits text into terms, dropping empty ones. Characters from scripts written without
/// spaces are split one by one; everything else stays in runs.
///
/// `"音質good"` becomes `["音", "質", "good"]`.
pub fn whitespace_tokenize(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for word in text.split(|c: char| c.is_whitespace() || TOKENIZE_DELIMITERS.contains(&c)) {
        if word.is_empty() {
            continue;
        }
        push_with_cjk_split(&mut out, word);
    }
    out
}

/// Adds one word, splitting the characters that need it and keeping the rest together.
fn push_with_cjk_split(out: &mut Vec<String>, word: &str) {
    let mut buf = String::new();
    for c in word.chars() {
        if is_cjk_nospace(c) {
            if !buf.is_empty() {
                out.push(std::mem::take(&mut buf));
            }
            out.push(c.to_string());
        } else {
            buf.push(c);
        }
    }
    if !buf.is_empty() {
        out.push(buf);
    }
}

/// Whether the text contains characters from a script written without spaces.
///
/// Judging a field by how many spaces it has would overlook exactly the text that needs
/// splitting most, so this exists to be asked instead.
pub fn has_cjk_nospace(text: &str) -> bool {
    text.chars().any(is_cjk_nospace)
}

/// Han characters and Japanese kana — the scripts written without spaces.
///
/// **Hangul is deliberately excluded.** Korean is written with spaces, so splitting it by
/// character would break words that were already correct.
fn is_cjk_nospace(c: char) -> bool {
    matches!(c,
        '\u{4E00}'..='\u{9FFF}'   // CJK Unified Ideographs
        | '\u{3400}'..='\u{4DBF}' // CJK Ext A
        | '\u{F900}'..='\u{FAFF}' // CJK Compatibility Ideographs
        | '\u{3040}'..='\u{309F}' // Hiragana
        | '\u{30A0}'..='\u{30FF}' // Katakana
        | '\u{31F0}'..='\u{31FF}' // Katakana Phonetic Extensions
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_on_whitespace() {
        assert_eq!(
            whitespace_tokenize("이 가방은 누구의"),
            ["이", "가방은", "누구의"]
        );
    }

    #[test]
    fn strips_trailing_punctuation() {
        assert_eq!(
            whitespace_tokenize("이 가방은 누구의 것입니까?"),
            ["이", "가방은", "누구의", "것입니까"]
        );
    }

    #[test]
    fn collapses_multiple_delimiters() {
        assert_eq!(whitespace_tokenize("a,  b ;; c"), ["a", "b", "c"]);
    }

    #[test]
    fn empty_input_yields_nothing() {
        assert!(whitespace_tokenize("").is_empty());
        assert!(whitespace_tokenize("   ").is_empty());
        assert!(whitespace_tokenize("?!,.").is_empty());
    }

    #[test]
    fn keeps_identifier_chars() {
        // These hold words together rather than ending them.
        assert_eq!(
            whitespace_tokenize("user@example-host"),
            ["user@example-host"]
        );
        assert_eq!(
            whitespace_tokenize("/var/log path_a"),
            ["/var/log", "path_a"]
        );
    }

    #[test]
    fn dot_is_a_boundary() {
        assert_eq!(whitespace_tokenize("v1.2.3"), ["v1", "2", "3"]);
    }

    #[test]
    fn hangul_not_char_split() {
        // Korean words survive whole.
        assert_eq!(whitespace_tokenize("이어폰"), ["이어폰"]);
    }

    #[test]
    fn han_and_kana_char_split() {
        // Without spaces to go on, characters are split individually.
        assert_eq!(whitespace_tokenize("音質"), ["音", "質"]);
        assert_eq!(whitespace_tokenize("東京"), ["東", "京"]);
        assert_eq!(
            whitespace_tokenize("ありがとう"),
            ["あ", "り", "が", "と", "う"]
        );
    }

    #[test]
    fn mixed_latin_and_han() {
        // Latin runs stay together; Han characters do not.
        assert_eq!(whitespace_tokenize("log音質ok"), ["log", "音", "質", "ok"]);
    }

    #[test]
    fn has_cjk_nospace_detects_han_kana_not_hangul_latin() {
        assert!(has_cjk_nospace("音質")); // Han
        assert!(has_cjk_nospace("ありがとう")); // kana
        assert!(has_cjk_nospace("log音質ok")); // mixed
        assert!(!has_cjk_nospace("이어폰 음질")); // Korean uses spaces
        assert!(!has_cjk_nospace("hello world")); // as does Latin
        assert!(!has_cjk_nospace(""));
    }
}
