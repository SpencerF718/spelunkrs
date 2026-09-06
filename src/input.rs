use std::path::Path;

/// Errors that can occur while acquiring text from an input.
#[derive(Debug)]
pub enum InputError {
    /// An underlying I/O error occurred.
    Io(std::io::Error),
    /// The file type is unsupported or does not have an implemented extractor.
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
    pub const TEXT_EXTENSIONS: &[&str] = &["md", "markdown", "txt", "srt", "vtt", "csv", "tsv"];

    /// Supported file extensions for audio and video media.
    pub const MEDIA_EXTENSIONS: &[&str] = &[
        "mp3", "wav", "m4a", "flac", "ogg", "opus", "aac", "wma", "mp4", "mkv", "webm", "mov",
        "avi",
    ];

    /// Supported file extensions for Anki flashcard decks and databases.
    pub const ANKI_EXTENSIONS: &[&str] = &["apkg", "colpkg", "anki2", "anki21"];

    /// Supported file extensions for PDF documents.
    pub const PDF_EXTENSIONS: &[&str] = &["pdf"];

    /// Supported file extensions for EPUB e-books.
    pub const EPUB_EXTENSIONS: &[&str] = &["epub"];

    /// Determines the [`FileType`] from a filesystem path by inspecting its extension.
    ///
    /// # Errors
    ///
    /// Returns [`InputError::UnsupportedFileType`] if the file has an unknown or
    /// unsupported extension.
    ///
    /// # Examples
    ///
    /// ```
    /// use spelunkrs::FileType;
    ///
    /// assert_eq!(FileType::from_path("note.md").unwrap(), FileType::Text);
    /// assert_eq!(FileType::from_path("audio.mp3").unwrap(), FileType::Media);
    /// assert!(FileType::from_path("program.exe").is_err());
    /// ```
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, InputError> {
        let extension = file_extension(path.as_ref());

        match extension.as_str() {
            extension_slice if Self::TEXT_EXTENSIONS.contains(&extension_slice) => {
                Ok(FileType::Text)
            }
            extension_slice if Self::MEDIA_EXTENSIONS.contains(&extension_slice) => {
                Ok(FileType::Media)
            }
            extension_slice if Self::ANKI_EXTENSIONS.contains(&extension_slice) => {
                Ok(FileType::Anki)
            }
            extension_slice if Self::PDF_EXTENSIONS.contains(&extension_slice) => Ok(FileType::Pdf),
            extension_slice if Self::EPUB_EXTENSIONS.contains(&extension_slice) => {
                Ok(FileType::Epub)
            }
            _ => Err(InputError::UnsupportedFileType(extension)),
        }
    }
}

/// Extracts raw text from a supported input file.
///
/// # Errors
///
/// Returns [`InputError::Io`] if the file cannot be read, or
/// [`InputError::UnsupportedFileType`] if the format has no text extractor.
pub fn extract_text(path: impl AsRef<Path>) -> Result<String, InputError> {
    let path = path.as_ref();

    match FileType::from_path(path)? {
        FileType::Text => std::fs::read_to_string(path).map_err(InputError::Io),
        _ => Err(InputError::UnsupportedFileType(file_extension(path))),
    }
}

fn file_extension(path: &Path) -> String {
    path.extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("")
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_text_preserves_markdown_formatting() {
        let path = "/tmp/test_raw_input.md";
        let content = "# 测试\n\n**测试**";

        std::fs::write(path, content).unwrap();
        let result = extract_text(path);
        let _ = std::fs::remove_file(path);

        assert_eq!(result.unwrap(), content);
    }

    #[test]
    fn extract_text_rejects_mp3_files() {
        let result = extract_text("audio.mp3");

        match result {
            Err(InputError::UnsupportedFileType(extension)) => {
                assert_eq!(extension, "mp3");
            }
            other => panic!("Expected UnsupportedFileType(\"mp3\"), got {other:?}"),
        }
    }

    #[test]
    fn from_path_returns_text_for_text_extensions() {
        for extension in FileType::TEXT_EXTENSIONS {
            let path = format!("file.{}", extension);
            let result = FileType::from_path(&path);
            assert_eq!(
                result.unwrap(),
                FileType::Text,
                "Expected Text for .{}",
                extension
            );
        }
    }

    #[test]
    fn from_path_returns_media_for_media_extensions() {
        for extension in FileType::MEDIA_EXTENSIONS {
            let path = format!("file.{}", extension);
            let result = FileType::from_path(&path);
            assert_eq!(
                result.unwrap(),
                FileType::Media,
                "Expected Media for .{}",
                extension
            );
        }
    }

    #[test]
    fn from_path_returns_anki_for_anki_extensions() {
        for extension in FileType::ANKI_EXTENSIONS {
            let path = format!("file.{}", extension);
            let result = FileType::from_path(&path);
            assert_eq!(
                result.unwrap(),
                FileType::Anki,
                "Expected Anki for .{}",
                extension
            );
        }
    }

    #[test]
    fn from_path_returns_pdf_for_pdf_extensions() {
        for extension in FileType::PDF_EXTENSIONS {
            let path = format!("document.{}", extension);
            let result = FileType::from_path(&path);
            assert_eq!(
                result.unwrap(),
                FileType::Pdf,
                "Expected Pdf for .{}",
                extension
            );
        }
    }

    #[test]
    fn from_path_returns_epub_for_epub_extensions() {
        for extension in FileType::EPUB_EXTENSIONS {
            let path = format!("novel.{}", extension);
            let result = FileType::from_path(&path);
            assert_eq!(
                result.unwrap(),
                FileType::Epub,
                "Expected Epub for .{}",
                extension
            );
        }
    }

    #[test]
    fn from_path_rejects_unsupported_extensions() {
        let result = FileType::from_path("archive.zip");
        match result {
            Err(InputError::UnsupportedFileType(extension)) => assert_eq!(extension, "zip"),
            _ => panic!("Expected UnsupportedFileType, got {:?}", result),
        }
    }

    #[test]
    fn from_path_rejects_paths_without_extensions() {
        let result = FileType::from_path("README");
        match result {
            Err(InputError::UnsupportedFileType(extension)) => assert_eq!(extension, ""),
            _ => panic!(
                "Expected UnsupportedFileType for no extension, got {:?}",
                result
            ),
        }
    }

    #[test]
    fn from_path_accepts_uppercase_extensions() {
        assert_eq!(FileType::from_path("NOTES.MD").unwrap(), FileType::Text);
        assert_eq!(FileType::from_path("audio.MP3").unwrap(), FileType::Media);
        assert_eq!(FileType::from_path("deck.APKG").unwrap(), FileType::Anki);
    }

    #[test]
    fn from_path_uses_last_extension_when_filename_has_multiple_dots() {
        let result = FileType::from_path("archive.tar.gz");
        match result {
            Err(InputError::UnsupportedFileType(extension)) => assert_eq!(extension, "gz"),
            _ => panic!("Expected UnsupportedFileType(\"gz\"), got {:?}", result),
        }
    }

    #[test]
    fn from_path_accepts_strings_and_path_types() {
        use std::path::{Path, PathBuf};

        let string_slice: &str = "test.md";
        let owned_string: String = String::from("test.md");
        let path_reference: &Path = Path::new("test.md");
        let path_buffer: PathBuf = PathBuf::from("test.md");

        assert_eq!(FileType::from_path(string_slice).unwrap(), FileType::Text);
        assert_eq!(FileType::from_path(&owned_string).unwrap(), FileType::Text);
        assert_eq!(FileType::from_path(path_reference).unwrap(), FileType::Text);
        assert_eq!(FileType::from_path(&path_buffer).unwrap(), FileType::Text);
        assert_eq!(FileType::from_path(path_buffer).unwrap(), FileType::Text);
    }
}
