//! # Spelunkrs
//!
//! A Chinese sentence-mining and vocabulary-profiling tool.

use jieba_rs::Jieba;
use std::collections::HashMap;
use std::path::Path;

/// Errors that can occur during tokenization and file loading.
#[derive(Debug)]
pub enum TokenizerError {
    /// An underlying I/O error occurred (e.g. file not found or permission denied).
    Io(std::io::Error),
    /// The file extension is not supported by the tokenizer.
    UnsupportedFileType(String),
}

/// Represents the supported file format categories for language input.
#[derive(Debug, PartialEq, Eq)]
pub enum FileType {
    /// Plain text, Markdown notes, or subtitle files (`.md`, `.txt`, `.srt`, `.vtt`, `.csv`, `.tsv`).
    Text,
    /// Audio or video files for speech transcription (`.mp3`, `.wav`, `.m4a`, `.mp4`, `.mkv`, etc.).
    Media,
    /// Anki flashcard deck packages or databases (`.apkg`, `.colpkg`, `.anki2`, `.anki21`).
    Anki,
    /// PDF documents for OCR or text extraction (`.pdf`).
    Pdf,
    /// EPUB e-books (`.epub`).
    Epub,
}

impl FileType {
    /// Supported file extensions for plain text, Markdown notes, and subtitles.
    pub const TEXT_EXTENSIONS: &'static [&'static str] = &[
        "md", "markdown", "txt", "srt", "vtt", "csv", "tsv",
    ];

    /// Supported file extensions for audio and video media.
    pub const MEDIA_EXTENSIONS: &'static [&'static str] = &[
        "mp3", "wav", "m4a", "flac", "ogg", "opus", "aac", "wma",
        "mp4", "mkv", "webm", "mov", "avi",
    ];

    /// Supported file extensions for Anki flashcard decks and databases.
    pub const ANKI_EXTENSIONS: &'static [&'static str] = &[
        "apkg", "colpkg", "anki2", "anki21",
    ];

    /// Supported file extensions for PDF documents.
    pub const PDF_EXTENSIONS: &'static [&'static str] = &["pdf"];

    /// Supported file extensions for EPUB e-books.
    pub const EPUB_EXTENSIONS: &'static [&'static str] = &["epub"];

    /// Determines the [`FileType`] from a filesystem path by inspecting its extension.
    ///
    /// # Errors
    ///
    /// Returns [`TokenizerError::UnsupportedFileType`] if the file has an unknown or
    /// unsupported extension.
    ///
    /// # Examples
    ///
    /// ```
    /// use spelunkrs::FileType;
    /// use std::path::Path;
    ///
    /// assert_eq!(FileType::from_path(Path::new("note.md")).unwrap(), FileType::Text);
    /// assert_eq!(FileType::from_path(Path::new("audio.mp3")).unwrap(), FileType::Media);
    /// assert!(FileType::from_path(Path::new("program.exe")).is_err());
    /// ```
    pub fn from_path(path: &Path) -> Result<Self, TokenizerError> {
        let extension = path
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            extension_slice if Self::TEXT_EXTENSIONS.contains(&extension_slice) => Ok(FileType::Text),
            extension_slice if Self::MEDIA_EXTENSIONS.contains(&extension_slice) => Ok(FileType::Media),
            extension_slice if Self::ANKI_EXTENSIONS.contains(&extension_slice) => Ok(FileType::Anki),
            extension_slice if Self::PDF_EXTENSIONS.contains(&extension_slice) => Ok(FileType::Pdf),
            extension_slice if Self::EPUB_EXTENSIONS.contains(&extension_slice) => Ok(FileType::Epub),
            _ => Err(TokenizerError::UnsupportedFileType(extension)),
        }
    }
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

    /// Reads a file from disk, detects its [`FileType`], and tokenizes Chinese vocabulary.
    ///
    /// # Supported Formats
    ///
    /// * **Plain text & Markdown**: `.md`, `.markdown`, `.txt`, `.srt`, `.vtt`, `.csv`, `.tsv`
    ///
    /// # Errors
    ///
    /// * [`TokenizerError::UnsupportedFileType`]: Returned if the file has an unsupported format.
    /// * [`TokenizerError::Io`]: Returned if the file cannot be read from disk (e.g. not found, permission denied).
    ///
    /// # Examples
    ///
    /// ```
    /// use spelunkrs::Tokenizer;
    ///
    /// let tokenizer = Tokenizer::new();
    /// let result = tokenizer.tokenize_file("nonexistent.md");
    /// assert!(result.is_err());
    /// ```
    pub fn tokenize_file(&self, path: impl AsRef<Path>) -> Result<HashMap<String, usize>, TokenizerError> {
        let path = path.as_ref();
        let file_type = FileType::from_path(path)?;

        match file_type {
            FileType::Text => {
                let content = std::fs::read_to_string(path).map_err(TokenizerError::Io)?;
                Ok(self.tokenize(&content))
            }
            FileType::Media => {
                todo!("Media (Whisper) transcription not yet implemented")
            }
            FileType::Anki => {
                todo!("Anki deck parsing not yet implemented")
            }
            FileType::Pdf => {
                todo!("PDF parsing not yet implemented")
            }
            FileType::Epub => {
                todo!("EPUB parsing not yet implemented")
            }
        }
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
    fn test_tokenizer_default() {
        let tokenizer = Tokenizer::default();
        let result = tokenizer.tokenize("测试");
        assert_eq!(result, HashMap::from([("测试".to_string(), 1)]));
    }

    #[test]
    fn test_tokenize_normal_sentence() {
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
    fn test_tokenize_punctuation_sentence() {
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
    fn test_tokenize_markdown_sentence() {
        let sentence = "# 汉字 \n **第一**";
        let expected: HashMap<String, usize> = HashMap::from([
            ("汉字".to_string(), 1),
            ("第一".to_string(), 1),
        ]);
        let tokenizer = Tokenizer::new();
        let result = tokenizer.tokenize(sentence);
        assert_eq!(result, expected);
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
    fn test_tokenize_whitespace_sentence() {
        let sentence = "   \n\t   ";
        let tokenizer = Tokenizer::new();
        let result = tokenizer.tokenize(sentence);
        assert!(result.is_empty());
    }

    #[test]
    fn test_tokenize_repeat_sentence() {
        let sentence = "你好你好你好";
        let expected: HashMap<String, usize> = HashMap::from([
            ("你好".to_string(), 3),
        ]);
        let tokenizer = Tokenizer::new();
        let result = tokenizer.tokenize(sentence);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tokenize_loanword_sentence() {
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
    fn test_tokenize_arabic_numerals() {
        let sentence = "2026年";
        let expected: HashMap<String, usize> = HashMap::from([
            ("年".to_string(), 1),
        ]);
        let tokenizer = Tokenizer::new();
        let result = tokenizer.tokenize(sentence);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tokenize_single_character() {
        let sentence = "王";
        let expected: HashMap<String, usize> = HashMap::from([
            ("王".to_string(), 1),
        ]);
        let tokenizer = Tokenizer::new();
        let result = tokenizer.tokenize(sentence);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tokenize_file_not_found() {
        let tokenizer = Tokenizer::new();
        let result = tokenizer.tokenize_file("fake_file.md");
        match result {
            Err(TokenizerError::Io(io_error)) => {
                assert_eq!(io_error.kind(), std::io::ErrorKind::NotFound);
            }
            _ => panic!("Expected TokenizerError::Io(NotFound), got {:?}", result),
        }
    }

    #[test]
    fn test_tokenize_file_markdown() {
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
        let result = tokenizer.tokenize_file(path);
        let _ = std::fs::remove_file(path);
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_tokenize_file_unsupported_file_type() {
        let tokenizer = Tokenizer::new();
        let result = tokenizer.tokenize_file("program.exe");

        match result {
            Err(TokenizerError::UnsupportedFileType(extension)) => {
                assert_eq!(extension, "exe");
            }
            _ => panic!("Expected UnsupportedFileType(\"exe\"), got {:?}", result),
        }
    }

    #[test]
    fn test_tokenize_file_empty_markdown() {
        let tokenizer = Tokenizer::new();
        let path = "/tmp/test_empty_file.md";
        std::fs::write(path, "").unwrap();
        let result = tokenizer.tokenize_file(path);
        let _ = std::fs::remove_file(path);
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_file_type_text_extensions() {
        for extension in FileType::TEXT_EXTENSIONS {
            let path = format!("file.{}", extension);
            let result = FileType::from_path(Path::new(&path));
            assert_eq!(result.unwrap(), FileType::Text, "Expected Text for .{}", extension);
        }
    }

    #[test]
    fn test_file_type_media_extensions() {
        for extension in FileType::MEDIA_EXTENSIONS {
            let path = format!("file.{}", extension);
            let result = FileType::from_path(Path::new(&path));
            assert_eq!(result.unwrap(), FileType::Media, "Expected Media for .{}", extension);
        }
    }

    #[test]
    fn test_file_type_anki_extensions() {
        for extension in FileType::ANKI_EXTENSIONS {
            let path = format!("file.{}", extension);
            let result = FileType::from_path(Path::new(&path));
            assert_eq!(result.unwrap(), FileType::Anki, "Expected Anki for .{}", extension);
        }
    }

    #[test]
    fn test_file_type_pdf() {
        for extension in FileType::PDF_EXTENSIONS {
            let path = format!("document.{}", extension);
            let result = FileType::from_path(Path::new(&path));
            assert_eq!(result.unwrap(), FileType::Pdf, "Expected Pdf for .{}", extension);
        }
    }

    #[test]
    fn test_file_type_epub() {
        for extension in FileType::EPUB_EXTENSIONS {
            let path = format!("novel.{}", extension);
            let result = FileType::from_path(Path::new(&path));
            assert_eq!(result.unwrap(), FileType::Epub, "Expected Epub for .{}", extension);
        }
    }

    #[test]
    fn test_file_type_unsupported() {
        let result = FileType::from_path(Path::new("archive.zip"));
        match result {
            Err(TokenizerError::UnsupportedFileType(extension)) => assert_eq!(extension, "zip"),
            _ => panic!("Expected UnsupportedFileType, got {:?}", result),
        }
    }

    #[test]
    fn test_file_type_no_extension() {
        let result = FileType::from_path(Path::new("README"));
        match result {
            Err(TokenizerError::UnsupportedFileType(extension)) => assert_eq!(extension, ""),
            _ => panic!("Expected UnsupportedFileType for no extension, got {:?}", result),
        }
    }

    #[test]
    fn test_file_type_uppercase_extension() {
        assert_eq!(FileType::from_path(Path::new("NOTES.MD")).unwrap(), FileType::Text);
        assert_eq!(FileType::from_path(Path::new("audio.MP3")).unwrap(), FileType::Media);
        assert_eq!(FileType::from_path(Path::new("deck.APKG")).unwrap(), FileType::Anki);
    }

    #[test]
    fn test_file_type_multiple_dots() {
        let result = FileType::from_path(Path::new("archive.tar.gz"));
        match result {
            Err(TokenizerError::UnsupportedFileType(extension)) => assert_eq!(extension, "gz"),
            _ => panic!("Expected UnsupportedFileType(\"gz\"), got {:?}", result),
        }
    }
}
