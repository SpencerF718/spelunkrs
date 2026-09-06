//! # Spelunkrs
//!
//! A Chinese sentence-mining and vocabulary-profiling tool.

use std::collections::HashMap;
use std::path::Path;

pub mod comparison;
pub mod input;
pub mod tokenizer;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_file_returns_not_found_when_file_is_missing() {
        let tokenizer = Tokenizer::new();
        let result = tokenize_file(&tokenizer, "fake_file.md");
        match result {
            Err(InputError::Io(io_error)) => {
                assert_eq!(io_error.kind(), std::io::ErrorKind::NotFound);
            }
            _ => panic!("Expected InputError::Io(NotFound), got {:?}", result),
        }
    }

    #[test]
    fn tokenize_file_counts_chinese_words_in_markdown() {
        let tokenizer = Tokenizer::new();
        let path = "/tmp/test_markdown_file.md";
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
        std::fs::write(path, content).unwrap();
        let expected = HashMap::from([
            ("测试".to_string(), 3),
            ("我们".to_string(), 1),
            ("来".to_string(), 1),
            ("一下".to_string(), 1),
            ("这个".to_string(), 1),
            ("系统".to_string(), 1),
        ]);
        let result = tokenize_file(&tokenizer, path);
        let _ = std::fs::remove_file(path);
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn tokenize_file_rejects_unsupported_file_types() {
        let tokenizer = Tokenizer::new();
        let result = tokenize_file(&tokenizer, "program.exe");

        match result {
            Err(InputError::UnsupportedFileType(extension)) => {
                assert_eq!(extension, "exe");
            }
            _ => panic!("Expected UnsupportedFileType(\"exe\"), got {:?}", result),
        }
    }

    #[test]
    fn tokenize_file_returns_no_words_when_markdown_file_is_empty() {
        let tokenizer = Tokenizer::new();
        let path = "/tmp/test_empty_file.md";
        std::fs::write(path, "").unwrap();
        let result = tokenize_file(&tokenizer, path);
        let _ = std::fs::remove_file(path);
        assert!(result.unwrap().is_empty());
    }
}
