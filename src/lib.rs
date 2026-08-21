//! # Spelunkrs
//!
//! A Chinese sentence-mining and vocabulary-profiling tool.

use jieba_rs::Jieba;

/// Returns 'true' if the char is a Chinese character.
///
/// Checks to see if the character falls between CJK Unified Ideographs
/// primary block range.  All whitespace, punctuation, and non-CJK characters
/// return false.
fn is_chinese(character: char) -> bool {
    matches!(character, '\u{4E00}'..='\u{9FFF}')
}

/// A Chinese word tokenizer backed by `jieba-rs`.
///
/// Strips whitespace, punctuation, and non-Chinese text, returning only valid Chinese words
/// along with a select few loanwords.
///
/// # Examples
///
/// ```
/// use spelunkrs::Tokenizer;
///
/// let tokenizer = Tokenizer::new();
/// let words = tokenizer.tokenize("我们来测试一下");
/// assert_eq!(words, vec!["我们", "来", "测试", "一下"]);
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
    /// Tokenizes a grouping of Chinese characters into a sequence
    /// of Chinese word.
    ///
    /// Non-Chinese characters, punctuation, and whitespace are excluded.
    ///
    /// # Examples
    ///
    /// ```
    /// use spelunkrs::Tokenizer;
    ///
    /// let tokenizer = Tokenizer::new();
    /// let words = tokenizer.tokenize("我们来测试一下");
    /// assert_eq!(words, vec!["我们", "来", "测试", "一下"]);
    /// ```
    pub fn tokenize<'a>(&self, sentence: &'a str) -> Vec<&'a str> {
        self.jieba
            .cut(sentence, false)
            .into_iter()
            .map(|token| token.word)
            .filter(|word| word.chars().any(is_chinese))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_chinese_bounds() {
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
    fn test_tokenize_normal_sentence() {
        let sentence = "我们来测试一下这个系统";
        let tokenizer = Tokenizer::new();
        let result = tokenizer.tokenize(sentence);
        assert_eq!(result, vec!["我们", "来", "测试", "一下", "这个", "系统"]);
    }

    #[test]
    fn test_tokenize_punctuation_sentence() {
        let sentence = "我们来测试一下这个系统，好吗？";
        let tokenizer = Tokenizer::new();
        let result = tokenizer.tokenize(sentence);
        assert_eq!(result, vec!["我们", "来", "测试", "一下", "这个", "系统", "好", "吗"]);
    }

    #[test]
    fn test_tokenize_markdown_sentence() {
        let sentence = "# 汉字 \n **第一**";
        let tokenizer = Tokenizer::new();
        let result = tokenizer.tokenize(sentence);
        assert_eq!(result, vec!["汉字", "第一"]);
    }

    #[test]
    fn test_tokenize_empty_sentence() {
        let sentence = "";
        let tokenizer = Tokenizer::new();
        let result = tokenizer.tokenize(sentence);
        assert!(result.is_empty());
    }

    #[test]
    fn test_tokenize_multilingual_sentence() {
        let sentence = "你好，我叫 Tim";
        let tokenizer = Tokenizer::new();
        let result = tokenizer.tokenize(sentence);
        assert_eq!(result, vec!["你好", "我", "叫"]);
    }

    #[test]
    fn test_tokenize_whitespace_sentence() {
        let sentence = "   \n\t   ";
        let tokenizer = Tokenizer::new();
        let result = tokenizer.tokenize(sentence);
        assert!(result.is_empty());
    }

    #[test]
    fn test_tokenize_repeat_sentence() {
        let sentence = "你好你好你好";
        let tokenizer = Tokenizer::new();
        let result = tokenizer.tokenize(sentence);
        assert_eq!(result, vec!["你好", "你好", "你好"]);
    }

    #[test]
    fn test_tokenize_loanword_sentence() {
        let sentence = "我们穿T恤去唱歌，这次AA制";
        let tokenizer = Tokenizer::new();
        let result = tokenizer.tokenize(sentence);
        assert_eq!(result, vec!["我们", "穿", "T恤", "去", "唱歌", "这次", "AA制"]);
    }

    #[test]
    fn test_tokenize_arabic_numerals() {
        let sentence = "2026年";
        let tokenizer = Tokenizer::new();
        let result = tokenizer.tokenize(sentence);
        assert_eq!(result, vec!["年"]);
    }

    #[test]
    fn test_tokenize_single_character() {
        let sentence = "王";
        let tokenizer = Tokenizer::new();
        let result = tokenizer.tokenize(sentence);
        assert_eq!(result, vec!["王"]);
    }
}
