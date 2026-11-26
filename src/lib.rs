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

/// Recursively tokenizes all supported files in `path` and returns their combined Chinese word counts.
///
/// Skips hidden entries, symbolic links, and unsupported file types inside the directory.
///
/// # Errors
///
/// Returns [`InputError::Io`] if a directory or supported file cannot be read.
///
/// # Examples
///
/// ```
/// use spelunkrs::{tokenize_directory, Tokenizer};
///
/// let tokenizer = Tokenizer::new();
/// let result = tokenize_directory(&tokenizer, "example_directory");
/// assert!(result.is_err());
/// ```
pub fn tokenize_directory(
    tokenizer: &Tokenizer,
    path: impl AsRef<Path>,
) -> Result<HashMap<String, usize>, InputError> {
    let mut total_counts = HashMap::new();
    walk_directory(tokenizer, path.as_ref(), &mut total_counts)?;
    Ok(total_counts)
}

/// Recursively tokenizes supported files under `directory` into `total_counts`.
fn walk_directory(
    tokenizer: &Tokenizer,
    directory: &Path,
    total_counts: &mut HashMap<String, usize>,
) -> Result<(), InputError> {
    let entries = std::fs::read_dir(directory)?;

    for entry in entries {
        let entry = entry?;
        let entry_path = entry.path();

        if input::is_hidden(&entry_path) {
            continue;
        }

        let entry_type = entry.file_type()?;

        if entry_type.is_dir() {
            walk_directory(tokenizer, &entry_path, total_counts)?;
        } else if entry_type.is_file() {
            match FileType::from_path(&entry_path) {
                Ok(file_type) if file_type.is_supported() => {
                    let file_counts = tokenize_file(tokenizer, &entry_path)?;
                    merge_counts(total_counts, file_counts);
                }
                Ok(_) | Err(InputError::UnsupportedFileType(_)) => {}
                Err(error) => return Err(error),
            }
        }
    }

    Ok(())
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

    #[test]
    fn tokenize_directory_returns_not_found_when_directory_is_missing() {
        let tokenizer = crate::test_tokenizer();
        let missing_dir =
            std::env::temp_dir().join(format!("tokenize_directory_missing-{}", std::process::id()));

        let result = tokenize_directory(tokenizer, &missing_dir);

        match result {
            Err(InputError::Io(io_error)) => {
                assert_eq!(io_error.kind(), std::io::ErrorKind::NotFound);
            }
            _ => panic!("Expected InputError::Io(NotFound), got {:?}", result),
        }
    }

    #[test]
    fn tokenize_directory_returns_error_when_path_is_a_file() {
        let tokenizer = crate::test_tokenizer();
        let file_path = std::env::temp_dir().join(format!(
            "tokenize_directory_returns_error_when_path_is_a_file-{}.md",
            std::process::id()
        ));
        std::fs::write(&file_path, "你好").unwrap();

        let result = tokenize_directory(tokenizer, &file_path);
        let _ = std::fs::remove_file(&file_path);

        assert!(
            matches!(result, Err(InputError::Io(_))),
            "Expected InputError::Io, got {:?}",
            result
        );
    }

    #[test]
    fn tokenize_directory_returns_empty_map_for_empty_directory() {
        let tokenizer = crate::test_tokenizer();
        let dir = std::env::temp_dir().join(format!(
            "tokenize_directory_returns_empty_map_for_empty_directory-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let result = tokenize_directory(tokenizer, &dir);
        let _ = std::fs::remove_dir_all(&dir);

        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn tokenize_directory_combines_counts_across_files_and_nested_subdirectories() {
        let tokenizer = crate::test_tokenizer();
        let dir = std::env::temp_dir().join(format!(
            "tokenize_directory_combines_counts_across_files_and_nested_subdirectories-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let nested_dir = dir.join("chapter1").join("notes");
        std::fs::create_dir_all(&nested_dir).unwrap();
        std::fs::write(dir.join("root.md"), "你好世界").unwrap();
        std::fs::write(dir.join("chapter1").join("vocab.txt"), "测试你好").unwrap();
        std::fs::write(nested_dir.join("deep.md"), "测试系统").unwrap();

        let result = tokenize_directory(tokenizer, &dir);
        let _ = std::fs::remove_dir_all(&dir);

        let expected = HashMap::from([
            ("你好".to_string(), 2),
            ("世界".to_string(), 1),
            ("测试".to_string(), 2),
            ("系统".to_string(), 1),
        ]);
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn tokenize_directory_skips_hidden_files_and_hidden_subdirectories() {
        let tokenizer = crate::test_tokenizer();
        let dir = std::env::temp_dir().join(format!(
            "tokenize_directory_skips_hidden_files_and_hidden_subdirectories-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let hidden_dir = dir.join(".obsidian");
        std::fs::create_dir_all(&hidden_dir).unwrap();
        std::fs::write(dir.join("visible.md"), "你好").unwrap();
        std::fs::write(dir.join(".hidden.md"), "世界").unwrap();
        std::fs::write(hidden_dir.join("secret.md"), "测试").unwrap();

        let result = tokenize_directory(tokenizer, &dir);
        let _ = std::fs::remove_dir_all(&dir);

        let expected = HashMap::from([("你好".to_string(), 1)]);
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn tokenize_directory_walks_hidden_root_while_skipping_hidden_children() {
        let tokenizer = crate::test_tokenizer();
        let dir = std::env::temp_dir().join(format!(
            ".tokenize_directory_hidden_root-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let visible_child = dir.join("notes");
        let hidden_child = dir.join(".obsidian");
        std::fs::create_dir_all(&visible_child).unwrap();
        std::fs::create_dir_all(&hidden_child).unwrap();
        std::fs::write(dir.join("root.md"), "你好世界").unwrap();
        std::fs::write(visible_child.join("child.md"), "你好测试").unwrap();
        std::fs::write(hidden_child.join("secret.md"), "系统").unwrap();

        let result = tokenize_directory(tokenizer, &dir);
        let _ = std::fs::remove_dir_all(&dir);

        let expected = HashMap::from([
            ("你好".to_string(), 2),
            ("世界".to_string(), 1),
            ("测试".to_string(), 1),
        ]);
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn tokenize_directory_skips_unsupported_and_unknown_files() {
        let tokenizer = crate::test_tokenizer();
        let dir = std::env::temp_dir().join(format!(
            "tokenize_directory_skips_unsupported_and_unknown_files-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("note.md"), "你好").unwrap();
        std::fs::write(dir.join("audio.mp3"), "世界").unwrap();
        std::fs::write(dir.join("book.pdf"), "测试").unwrap();
        std::fs::write(dir.join("program.exe"), "系统").unwrap();
        std::fs::write(dir.join("README"), "我们").unwrap();

        let result = tokenize_directory(tokenizer, &dir);
        let _ = std::fs::remove_dir_all(&dir);

        let expected = HashMap::from([("你好".to_string(), 1)]);
        assert_eq!(result.unwrap(), expected);
    }

    #[cfg(unix)]
    #[test]
    fn tokenize_directory_skips_symlinks() {
        let tokenizer = crate::test_tokenizer();
        let dir = std::env::temp_dir().join(format!(
            "tokenize_directory_skips_symlinks-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let real_file = dir.join("note.md");
        std::fs::write(&real_file, "你好").unwrap();
        std::os::unix::fs::symlink(&real_file, dir.join("linked.md")).unwrap();
        std::os::unix::fs::symlink(&dir, dir.join("loop")).unwrap();

        let result = tokenize_directory(tokenizer, &dir);
        let _ = std::fs::remove_dir_all(&dir);

        let expected = HashMap::from([("你好".to_string(), 1)]);
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn tokenize_directory_returns_io_error_for_invalid_utf8_file() {
        let tokenizer = crate::test_tokenizer();
        let dir = std::env::temp_dir().join(format!(
            "tokenize_directory_returns_io_error_for_invalid_utf8_file-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("bad.txt"), [0xFF, 0xFE]).unwrap();

        let result = tokenize_directory(tokenizer, &dir);
        let _ = std::fs::remove_dir_all(&dir);

        match result {
            Err(InputError::Io(io_error)) => {
                assert_eq!(io_error.kind(), std::io::ErrorKind::InvalidData);
            }
            _ => panic!("Expected InputError::Io(InvalidData), got {:?}", result),
        }
    }
}
