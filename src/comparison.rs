use std::collections::HashMap;

/// Returns words from `target` that are absent from the selected `known` corpus.
///
/// Each result contains an owned word and its occurrence count from `target`.
/// Results are ordered by descending target count, with ascending word order
/// used to break ties.
///
/// # Examples
///
/// ```
/// use spelunkrs::comparison::target_specific_words;
/// use std::collections::HashMap;
///
/// let known = HashMap::from([
///     ("你好".to_string(), 10),
/// ]);
/// let target = HashMap::from([
///     ("你好".to_string(), 2),
///     ("世界".to_string(), 3),
/// ]);
///
/// let result = target_specific_words(&known, &target);
///
/// assert_eq!(result, vec![("世界".to_string(), 3)]);
/// ```
pub fn target_specific_words(
    known: &HashMap<String, usize>,
    target: &HashMap<String, usize>,
) -> Vec<(String, usize)> {
    let mut words: Vec<(String, usize)> = target
        .iter()
        .filter(|&(word, _count)| !known.contains_key(word))
        .map(|(word, count)| (word.clone(), *count))
        .collect();

    words.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    words
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_specific_words_returns_empty_when_both_maps_are_empty() {
        let known: HashMap<String, usize> = HashMap::new();
        let target: HashMap<String, usize> = HashMap::new();

        let result = target_specific_words(&known, &target);

        assert!(result.is_empty());
    }

    #[test]
    fn target_specific_words_returns_empty_when_target_is_empty() {
        let known: HashMap<String, usize> =
            HashMap::from([("你好".to_string(), 3), ("世界".to_string(), 1)]);
        let target: HashMap<String, usize> = HashMap::new();

        let result = target_specific_words(&known, &target);

        assert!(result.is_empty());
    }

    #[test]
    fn target_specific_words_returns_target_word_with_its_count_when_known_is_empty() {
        let known: HashMap<String, usize> = HashMap::new();
        let target: HashMap<String, usize> = HashMap::from([("你好".to_string(), 3)]);

        let expected = vec![("你好".to_string(), 3)];
        let result = target_specific_words(&known, &target);

        assert_eq!(result, expected);
    }

    #[test]
    fn target_specific_words_returns_empty_when_all_target_words_are_known() {
        let known: HashMap<String, usize> = HashMap::from([("你好".to_string(), 99)]);
        let target: HashMap<String, usize> = HashMap::from([("你好".to_string(), 3)]);

        let result = target_specific_words(&known, &target);

        assert!(result.is_empty());
    }

    #[test]
    fn target_specific_words_returns_unknown_words_with_target_counts_when_some_words_are_known() {
        let known: HashMap<String, usize> = HashMap::from([("你好".to_string(), 99)]);
        let target: HashMap<String, usize> =
            HashMap::from([("你好".to_string(), 3), ("世界".to_string(), 1)]);

        let expected = vec![("世界".to_string(), 1)];
        let result = target_specific_words(&known, &target);

        assert_eq!(result, expected);
    }

    #[test]
    fn target_specific_words_sorts_by_descending_count() {
        let known: HashMap<String, usize> = HashMap::new();
        let target: HashMap<String, usize> =
            HashMap::from([("你好".to_string(), 3), ("世界".to_string(), 1)]);

        let expected = vec![("你好".to_string(), 3), ("世界".to_string(), 1)];
        let result = target_specific_words(&known, &target);

        assert_eq!(result, expected);
    }

    #[test]
    fn target_specific_words_sorts_by_word_when_counts_are_equal() {
        let known: HashMap<String, usize> = HashMap::new();
        let target: HashMap<String, usize> =
            HashMap::from([("你好".to_string(), 3), ("世界".to_string(), 3)]);

        let expected = vec![("世界".to_string(), 3), ("你好".to_string(), 3)];
        let result = target_specific_words(&known, &target);

        assert_eq!(result, expected);
    }

    #[test]
    fn target_specific_words_filters_known_words_and_sorts_by_count_then_word() {
        let known: HashMap<String, usize> = HashMap::from([("五".to_string(), 10)]);
        let target: HashMap<String, usize> = HashMap::from([
            ("一".to_string(), 5),
            ("二".to_string(), 5),
            ("三".to_string(), 3),
            ("四".to_string(), 1),
            ("五".to_string(), 99),
        ]);

        let expected = vec![
            ("一".to_string(), 5),
            ("二".to_string(), 5),
            ("三".to_string(), 3),
            ("四".to_string(), 1),
        ];
        let result = target_specific_words(&known, &target);

        assert_eq!(result, expected);
    }
}
