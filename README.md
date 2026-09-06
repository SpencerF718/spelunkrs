# spelunkrs
A lightweight Rust CLI for Chinese sentence mining across local Markdown vaults.

## Features

- [x] **Chinese Word Segmentation:** Split Chinese text into words via [jieba-rs](https://github.com/messense/jieba-rs).
- [x] **Accurate Text Filtering:** Automatically filters out punctuation, whitespace, and non-Chinese words.
- [x] **Loanword Support:** Preserves mixed Chinese and Latin words such as `T恤` and `AA制`.
- [x] **Word Occurrence Counting:** Count occurrences of each word in text or a supported file.
- [x] **Text File Input:** Read Markdown, plain text, SRT, VTT, CSV, and TSV files as raw text.
- [x] **Vocabulary Comparison:** Compare known and target word counts through the library to find words absent from the known vocabulary, sorted by descending occurrence count with ties ordered by word.

## Coming Soon

- [ ] **Additional Input Formats:** Extract text from PDF and EPUB documents, Anki decks, and audio/video files through transcription.
- [ ] **Vocabulary Statistics:** Explore vocabulary usage and knowledge through statistics and insights.
- [ ] **Custom Dictionaries:** Add custom words, slang, and other vocabulary.
- [ ] **Traditional Chinese Support:** Tokenization support for Traditional Chinese text.
