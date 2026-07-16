//! kaflow-tokenizer — 검색 term 토크나이저 **인터페이스 + 경량 기본 구현** (public, 의존성 0).
//!
//! 토큰화는 인덱싱 I-key 발급과 검색어 분해가 공유하는 chokepoint 다. 이 크레이트는 그
//! **계약([`Tokenizer`])** 과 **사전 없는 기본 구현([`WhitespaceTokenizer`])** 만 담는다.
//!
//! ## public / private 분리
//! - **public(여기)**: 인터페이스 + 가벼운 [`WhitespaceTokenizer`]. 무거운 의존성 0 →
//!   public 소비자(mock 데모 / 외부 엔진 구현)가 부담 없이 쓴다.
//! - **private(엔진)**: 형태소 분절(charabia + CJK 사전) `CharabiaTokenizer` 가 이 trait 를
//!   구현한다. 무거운 사전은 private 에만 둔다.
//!
//! 어느 구현체를 쓸지는 **빌드(크레이트) 단위**로 갈린다 — 실엔진은 charabia, mock·폴백은
//! whitespace. 살아있는 인덱스를 두고 런타임에 토글하는 용도가 아니다(전환 = 기존 인덱스 stale).

/// 토크나이저 구현체 공통 계약. 인덱싱 I-key 발급 / 검색어 분해가 공유하는 단일 chokepoint
/// 이므로, 어떤 구현체를 끼우든 **인덱싱과 검색이 같은 함수를 통과**한다 (drift 원천 차단).
pub trait Tokenizer {
    /// 텍스트를 검색 term 단위로 쪼갠다. 빈 토큰은 포함하지 않는다.
    fn tokenize(&self, text: &str) -> Vec<String>;
}

/// 어절 경계로 쓰는 구두점. 공백류는 `char::is_whitespace` 로 별도 판정한다.
///
/// `@ - _ /` 는 이메일 / URL / 코드 / 경로에서 단어의 일부라 의도적으로 제외한다.
/// `.` 은 문장부호 겸용이라 일단 경계에 포함하지만, 식별자(`v1.2.3` 등) 검색에 영향이
/// 있어 추후 실데이터로 조정할 수 있다. 한 곳에서 관리하도록 상수로 분리.
pub const TOKENIZE_DELIMITERS: &[char] = &[
    '?', '!', ',', '.', ';', ':', '"', '\'', '(', ')', '[', ']', '{', '}',
];

/// 공백 + 구두점 분할 토크나이저 (경량 기본 구현). + **무사전 CJK 폴백**.
///
/// 한국어/영어는 공백을 쓰므로 공백 분할로 충분하다. 일본어/중국어는 공백이 없어 한 덩어리로
/// 떨어지므로, **사전 없이** 한자(Han)·가나(Kana) 글자를 1글자 토큰으로 쪼갠다(아래
/// [`whitespace_tokenize`]). 정밀하진 않지만 deps 0 · 결정론이라 mock 데모/경량 소비자에 충분.
/// 정밀 CJK 분절(형태소·기능어 제거)은 private `CharabiaTokenizer` 가 담당한다.
#[derive(Debug, Default, Clone, Copy)]
pub struct WhitespaceTokenizer;

impl Tokenizer for WhitespaceTokenizer {
    fn tokenize(&self, text: &str) -> Vec<String> {
        whitespace_tokenize(text)
    }
}

/// 텍스트를 어절 단위로 쪼갠다. 공백류 + [`TOKENIZE_DELIMITERS`] 가 경계이며, 빈 토큰은
/// 제거된다. 어절 안의 **한자·가나 글자는 1글자씩** 더 쪼갠다(공백 없는 CJK 폴백).
///
/// 예: `"이 가방은 누구의 것입니까?"` → `["이", "가방은", "누구의", "것입니까"]`
/// 예: `"音質good"` → `["音", "質", "good"]`  (한자는 글자 단위, 라틴은 묶음)
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

/// 한 어절을 토큰으로 push 하되, 공백 없는 CJK(한자·가나) 글자는 1글자씩 쪼개고 나머지
/// (한글·라틴·숫자 등)는 연속 run 으로 묶는다.
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

/// 텍스트가 **공백 없는 CJK(한자·가나)** 글자를 하나라도 포함하는지.
///
/// 어절(tokenize) 후보 추천이 "공백 다단어"만 보면 일/중(공백 없음) 텍스트를 놓치므로, 이
/// stable 술어를 공유해 "CJK 텍스트도 토큰화 이득 필드"로 잡게 한다 (codepoint 범위는 불변).
pub fn has_cjk_nospace(text: &str) -> bool {
    text.chars().any(is_cjk_nospace)
}

/// 공백을 쓰지 않는 CJK script — 한자(Han) + 일본어 가나(Hiragana/Katakana).
/// **한글(Hangul)은 제외**한다 — 한국어는 공백을 쓰므로 어절 분할로 충분(글자 분해는 오히려 해롭다).
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
        // @ - _ / 는 경계가 아니므로 토큰에 남는다 (라틴은 묶음 유지).
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
        // 한글은 어절 그대로 (글자 분해 안 함).
        assert_eq!(whitespace_tokenize("이어폰"), ["이어폰"]);
    }

    #[test]
    fn han_and_kana_char_split() {
        // 공백 없는 일/중은 글자 단위로 쪼갠다 (무사전 폴백).
        assert_eq!(whitespace_tokenize("音質"), ["音", "質"]);
        assert_eq!(whitespace_tokenize("東京"), ["東", "京"]);
        assert_eq!(
            whitespace_tokenize("ありがとう"),
            ["あ", "り", "が", "と", "う"]
        );
    }

    #[test]
    fn mixed_latin_and_han() {
        // 라틴 run 은 묶고, 한자는 글자 단위.
        assert_eq!(whitespace_tokenize("log音質ok"), ["log", "音", "質", "ok"]);
    }

    #[test]
    fn has_cjk_nospace_detects_han_kana_not_hangul_latin() {
        assert!(has_cjk_nospace("音質")); // 한자
        assert!(has_cjk_nospace("ありがとう")); // 가나
        assert!(has_cjk_nospace("log音質ok")); // 혼합
        assert!(!has_cjk_nospace("이어폰 음질")); // 한글(공백 사용) → false
        assert!(!has_cjk_nospace("hello world")); // 라틴 → false
        assert!(!has_cjk_nospace("")); // 빈 문자열
    }
}
