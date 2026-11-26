//! # Spelunkrs
//!
//! A Chinese sentence-mining and vocabulary-profiling tool.

use std::collections::HashMap;
use std::path::Path;

pub mod comparison;
pub mod input;
pub mod tokenizer;

#[cfg(test)]
pub(crate) fn test_tokenizer() -> &'static Tokenizer {
    static TOKENIZER: std::sync::LazyLock<Tokenizer> = std::sync::LazyLock::new(Tokenizer::new);

    std::sync::LazyLock::force(&TOKENIZER)
}

pub use input::{FileType, InputError};
pub use tokenizer::Tokenizer;

/// Extracts text from a file and returns its Chinese word occurrence counts.
///
/// # Supported Formats
///
/// * **Plain text & Markdown**: `.md`, `.markdown`, `.txt`, `.srt`, `.vtt`, `.csv`, `.tsv`
///
/// # Errors
///
/// * [`InputError::UnsupportedFileType`]: Returned if the file has an unsupported format.
/// * [`InputError::Io`]: Returned if the file cannot be read from disk.
///
/// # Examples
///
/// ```
/// use spelunkrs::{tokenize_file, Tokenizer};
///
/// let tokenizer = Tokenizer::new();
/// let result = tokenize_file(&tokenizer, "nonexistent.md");
/// assert!(result.is_err());
/// ```
pub fn tokenize_file(
    tokenizer: &Tokenizer,
    path: impl AsRef<Path>,
) -> Result<HashMap<String, usize>, InputError> {
    let text = input::extract_text(path)?;
    Ok(tokenizer.tokenize(&text))
}

/// Adds all word counts from `new_counts` into `total_counts`.
fn merge_counts(total_counts: &mut HashMap<String, usize>, new_counts: HashMap<String, usize>) {
    for (word, count) in new_counts {
        let total = total_counts.entry(word).or_insert(0);
        *total += count;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_file_returns_not_found_when_file_is_missing() {
        let tokenizer = crate::test_tokenizer();
        let result = tokenize_file(tokenizer, "fake_file.md");
        match result {
            Err(InputError::Io(io_error)) => {
                assert_eq!(io_error.kind(), std::io::ErrorKind::NotFound);
            }
            _ => panic!("Expected InputError::Io(NotFound), got {:?}", result),
        }
    }

    #[test]
    fn tokenize_file_counts_chinese_words_in_markdown() {
        let tokenizer = crate::test_tokenizer();
        let path = std::env::temp_dir().join(format!(
            "tokenize_file_counts_chinese_words_in_markdown-{}.md",
            std::process::id()
        ));
        let content = "
---
tags:
- chinese_vocab
- sentence_mined
---
# 测试

## 我们来测试一下这个系统。 #card-reverse
Target Word: 测试 (ce4 shi4) - to test
Translation: Let's test out this system.
";
        std::fs::write(&path, content).unwrap();
        let expected = HashMap::from([
            ("测试".to_string(), 3),
            ("我们".to_string(), 1),
            ("来".to_string(), 1),
            ("一下".to_string(), 1),
            ("这个".to_string(), 1),
            ("系统".to_string(), 1),
        ]);
        let result = tokenize_file(tokenizer, &path);
        let _ = std::fs::remove_file(&path);
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn tokenize_file_rejects_unsupported_file_types() {
        let tokenizer = crate::test_tokenizer();
        let result = tokenize_file(tokenizer, "program.exe");

        match result {
            Err(InputError::UnsupportedFileType(extension)) => {
                assert_eq!(extension, "exe");
            }
            _ => panic!("Expected UnsupportedFileType(\"exe\"), got {:?}", result),
        }
    }

    #[test]
    fn tokenize_file_returns_no_words_when_markdown_file_is_empty() {
        let tokenizer = crate::test_tokenizer();
        let path = std::env::temp_dir().join(format!(
            "tokenize_file_returns_no_words_when_markdown_file_is_empty-{}.md",
            std::process::id()
        ));
        std::fs::write(&path, "").unwrap();
        let result = tokenize_file(tokenizer, &path);
        let _ = std::fs::remove_file(&path);
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn merge_counts_combines_overlapping_and_distinct_words() {
        let mut total_counts = HashMap::from([("你好".to_string(), 2), ("世界".to_string(), 1)]);
        let new_counts = HashMap::from([("世界".to_string(), 3), ("测试".to_string(), 4)]);

        merge_counts(&mut total_counts, new_counts);

        let expected = HashMap::from([
            ("你好".to_string(), 2),
            ("世界".to_string(), 4),
            ("测试".to_string(), 4),
        ]);
        assert_eq!(total_counts, expected);
    }

    #[test]
    fn merge_counts_inserts_all_words_when_total_is_empty() {
        let mut total_counts = HashMap::new();
        let new_counts = HashMap::from([("你好".to_string(), 2), ("世界".to_string(), 5)]);

        merge_counts(&mut total_counts, new_counts);

        let expected = HashMap::from([("你好".to_string(), 2), ("世界".to_string(), 5)]);
        assert_eq!(total_counts, expected);
    }

    #[test]
    fn merge_counts_preserves_total_when_new_counts_is_empty() {
        let mut total_counts = HashMap::from([("你好".to_string(), 2), ("世界".to_string(), 1)]);
        let new_counts = HashMap::new();

        merge_counts(&mut total_counts, new_counts);

        let expected = HashMap::from([("你好".to_string(), 2), ("世界".to_string(), 1)]);
        assert_eq!(total_counts, expected);
    }

    #[test]
    fn merge_counts_stays_empty_when_both_maps_are_empty() {
        let mut total_counts = HashMap::new();
        let new_counts = HashMap::new();

        merge_counts(&mut total_counts, new_counts);

        assert!(total_counts.is_empty());
    }

    #[test]
    fn merge_counts_accumulates_across_multiple_merges() {
        let mut total_counts = HashMap::new();
        let first = HashMap::from([("测试".to_string(), 1), ("我们".to_string(), 2)]);
        let second = HashMap::from([("测试".to_string(), 3), ("系统".to_string(), 1)]);
        let third = HashMap::from([("测试".to_string(), 2), ("我们".to_string(), 1)]);

        merge_counts(&mut total_counts, first);
        merge_counts(&mut total_counts, second);
        merge_counts(&mut total_counts, third);

        let expected = HashMap::from([
            ("测试".to_string(), 6),
            ("我们".to_string(), 3),
            ("系统".to_string(), 1),
        ]);
        assert_eq!(total_counts, expected);
    }
}
