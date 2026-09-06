use jieba_rs::Jieba;
use std::collections::HashMap;

/// A Chinese word tokenizer backed by `jieba-rs`.
///
/// Strips whitespace, punctuation, and non-Chinese text, returning only valid Chinese words
/// along with a select few loanwords.
///
/// # Examples
///
/// ```
/// use spelunkrs::Tokenizer;
/// use std::collections::HashMap;
///
/// let tokenizer = Tokenizer::new();
/// let words = tokenizer.tokenize("我们来测试一下");
/// let expected = HashMap::from([
///     ("我们".to_string(), 1),
///     ("来".to_string(), 1),
///     ("测试".to_string(), 1),
///     ("一下".to_string(), 1),
/// ]);
/// assert_eq!(words, expected);
/// ```
pub struct Tokenizer {
    jieba: Jieba,
}

impl Tokenizer {
    /// Creates a new 'Tokenizer' and initializes the jieba dictionary.
    pub fn new() -> Self {
        Self {
            jieba: Jieba::new(),
        }
    }

    /// Tokenizes Chinese text and returns a map of unique Chinese words
    /// and their occurrence counts.
    ///
    /// Non-Chinese characters, punctuation, and whitespace are excluded.
    ///
    /// # Examples
    ///
    /// ```
    /// use spelunkrs::Tokenizer;
    /// use std::collections::HashMap;
    ///
    /// let tokenizer = Tokenizer::new();
    /// let words = tokenizer.tokenize("我们来测试一下");
    /// let expected = HashMap::from([
    ///     ("我们".to_string(), 1),
    ///     ("来".to_string(), 1),
    ///     ("测试".to_string(), 1),
    ///     ("一下".to_string(), 1),
    /// ]);
    /// assert_eq!(words, expected);
    /// ```
    pub fn tokenize(&self, sentence: &str) -> HashMap<String, usize> {
        self.jieba
            .cut(sentence, false)
            .into_iter()
            .map(|token| token.word)
            .filter(|word| word.chars().any(is_chinese))
            .fold(HashMap::new(), |mut map, word| {
                *map.entry(word.to_string()).or_insert(0) += 1;
                map
            })
    }
}

impl Default for Tokenizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns 'true' if the char is a Chinese character.
///
/// Checks to see if the character falls between CJK Unified Ideographs
/// primary block range.  All whitespace, punctuation, and non-CJK characters
/// return false.
fn is_chinese(character: char) -> bool {
    matches!(character, '\u{4E00}'..='\u{9FFF}')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_chinese_accepts_only_characters_in_the_basic_cjk_range() {
        assert!(is_chinese('一'));
        assert!(is_chinese('鿿'));
        assert!(is_chinese('我'));

        assert!(!is_chinese('\u{4DFF}'));
        assert!(!is_chinese('\u{A000}'));
        assert!(!is_chinese('A'));
        assert!(!is_chinese('.'));
        assert!(!is_chinese('\n'));
    }

    #[test]
    fn default_creates_tokenizer_that_segments_chinese_words() {
        let tokenizer = Tokenizer::default();
        let result = tokenizer.tokenize("测试");
        assert_eq!(result, HashMap::from([("测试".to_string(), 1)]));
    }

    #[test]
    fn tokenize_segments_chinese_words() {
        let sentence = "我们来测试一下这个系统";
        let expected: HashMap<String, usize> = HashMap::from([
            ("我们".to_string(), 1),
            ("来".to_string(), 1),
            ("测试".to_string(), 1),
            ("一下".to_string(), 1),
            ("这个".to_string(), 1),
            ("系统".to_string(), 1),
        ]);
        let tokenizer = Tokenizer::new();
        let result = tokenizer.tokenize(sentence);
        assert_eq!(result, expected);
    }

    #[test]
    fn tokenize_ignores_punctuation() {
        let sentence = "我们来测试一下这个系统，好吗？";
        let expected: HashMap<String, usize> = HashMap::from([
            ("我们".to_string(), 1),
            ("来".to_string(), 1),
            ("测试".to_string(), 1),
            ("一下".to_string(), 1),
            ("这个".to_string(), 1),
            ("系统".to_string(), 1),
            ("好".to_string(), 1),
            ("吗".to_string(), 1),
        ]);
        let tokenizer = Tokenizer::new();
        let result = tokenizer.tokenize(sentence);
        assert_eq!(result, expected);
    }

    #[test]
    fn tokenize_ignores_markdown_markers() {
        let sentence = "# 汉字 \n **第一**";
        let expected: HashMap<String, usize> =
            HashMap::from([("汉字".to_string(), 1), ("第一".to_string(), 1)]);
        let tokenizer = Tokenizer::new();
        let result = tokenizer.tokenize(sentence);
        assert_eq!(result, expected);
    }

    #[test]
    fn tokenize_returns_no_words_when_input_is_empty() {
        let sentence = "";
        let tokenizer = Tokenizer::new();
        let result = tokenizer.tokenize(sentence);
        assert!(result.is_empty());
    }

    #[test]
    fn tokenize_ignores_english_words_when_mixed_with_chinese() {
        let sentence = "你好，我叫 Tim";
        let expected: HashMap<String, usize> = HashMap::from([
            ("你好".to_string(), 1),
            ("我".to_string(), 1),
            ("叫".to_string(), 1),
        ]);
        let tokenizer = Tokenizer::new();
        let result = tokenizer.tokenize(sentence);
        assert_eq!(result, expected);
    }

    #[test]
    fn tokenize_returns_no_words_when_input_is_whitespace() {
        let sentence = "   \n\t   ";
        let tokenizer = Tokenizer::new();
        let result = tokenizer.tokenize(sentence);
        assert!(result.is_empty());
    }

    #[test]
    fn tokenize_counts_repeated_words() {
        let sentence = "你好你好你好";
        let expected: HashMap<String, usize> = HashMap::from([("你好".to_string(), 3)]);
        let tokenizer = Tokenizer::new();
        let result = tokenizer.tokenize(sentence);
        assert_eq!(result, expected);
    }

    #[test]
    fn tokenize_preserves_loanwords_containing_latin_letters() {
        let sentence = "我们穿T恤去唱歌，这次AA制";
        let expected: HashMap<String, usize> = HashMap::from([
            ("我们".to_string(), 1),
            ("穿".to_string(), 1),
            ("T恤".to_string(), 1),
            ("去".to_string(), 1),
            ("唱歌".to_string(), 1),
            ("这次".to_string(), 1),
            ("AA制".to_string(), 1),
        ]);
        let tokenizer = Tokenizer::new();
        let result = tokenizer.tokenize(sentence);
        assert_eq!(result, expected);
    }

    #[test]
    fn tokenize_ignores_arabic_numerals() {
        let sentence = "2026年";
        let expected: HashMap<String, usize> = HashMap::from([("年".to_string(), 1)]);
        let tokenizer = Tokenizer::new();
        let result = tokenizer.tokenize(sentence);
        assert_eq!(result, expected);
    }

    #[test]
    fn tokenize_returns_one_word_when_input_is_a_single_chinese_character() {
        let sentence = "王";
        let expected: HashMap<String, usize> = HashMap::from([("王".to_string(), 1)]);
        let tokenizer = Tokenizer::new();
        let result = tokenizer.tokenize(sentence);
        assert_eq!(result, expected);
    }
}
