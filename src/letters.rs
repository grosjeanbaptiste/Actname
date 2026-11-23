//! A letter model trained on the words of the semantic base, to propose
//! letter sequences steered by a request.
//!
//! Deterministic: integer counts, no randomness, no floating point; ties break
//! on the sequence itself, so the same base and request give the same names.
//!
//! The model counts, for every two letters seen in a word of the base, which
//! letter follows (`$` marks the end of a word). A search keeps the `BEAM`
//! best sequences letter by letter: each step adds the permille probability of
//! the letter after the last two, plus `BIAS` when the suffix built so far
//! occurs inside a word of the request's concepts.

use std::collections::{BTreeMap, BTreeSet};

/// Shortest suffix that may stand as a name part.
const MIN_SUFFIX: usize = 3;
/// Sequences kept at each step.
const BEAM: usize = 256;
/// Bonus per letter while the suffix stays inside a word of the request.
const BIAS: i64 = 1000;
const END: char = '$';
const START: char = '^';

#[derive(Debug, Default)]
pub struct LetterModel {
    /// Last two letters → letter that follows → count.
    next: BTreeMap<String, BTreeMap<char, u32>>,
}

fn last_two(text: &str) -> String {
    let letters: Vec<char> = text.chars().collect();
    letters[letters.len().saturating_sub(2)..].iter().collect()
}

impl LetterModel {
    /// Counts the letters of every lowercase ASCII word of three letters or more.
    pub fn train<'a>(words: impl IntoIterator<Item = &'a str>) -> Self {
        let mut model = Self::default();
        for word in words {
            if word.len() < 3 || !word.chars().all(|c| c.is_ascii_lowercase()) {
                continue;
            }
            let mut seen = String::from(START);
            for letter in word.chars().chain(std::iter::once(END)) {
                *model
                    .next
                    .entry(last_two(&seen))
                    .or_default()
                    .entry(letter)
                    .or_default() += 1;
                seen.push(letter);
            }
        }
        model
    }

    /// Names that start with `prefix`, at most `max_length` letters, best first.
    pub fn names(
        &self,
        prefix: &str,
        anchors: &BTreeSet<String>,
        max_length: usize,
    ) -> Vec<String> {
        let mut beam: Vec<(i64, String)> = vec![(0, String::new())];
        let mut done: BTreeMap<String, i64> = BTreeMap::new();
        for _ in prefix.len()..max_length {
            let mut grown: Vec<(i64, String)> = Vec::new();
            for (score, suffix) in &beam {
                // The suffix reads like a word of its own: its context starts at its first letter.
                let context = last_two(&format!("{START}{suffix}"));
                let Some(counts) = self.next.get(&context) else {
                    continue;
                };
                let total: u32 = counts.values().sum();
                for (&letter, &count) in counts {
                    let step = i64::from(count) * 1000 / i64::from(total);
                    if letter == END {
                        if suffix.len() >= MIN_SUFFIX {
                            let best = done.entry(format!("{prefix}{suffix}")).or_insert(i64::MIN);
                            *best = (*best).max(score + step);
                        }
                        continue;
                    }
                    let mut longer = suffix.clone();
                    longer.push(letter);
                    let bias = if longer.len() >= 2 && anchors.iter().any(|w| w.contains(&longer)) {
                        BIAS
                    } else {
                        0
                    };
                    grown.push((score + step + bias, longer));
                }
            }
            grown.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
            grown.truncate(BEAM);
            beam = grown;
        }
        for (score, suffix) in beam {
            if suffix.len() >= MIN_SUFFIX {
                let best = done.entry(format!("{prefix}{suffix}")).or_insert(i64::MIN);
                *best = (*best).max(score);
            }
        }
        let mut ranked: Vec<(i64, String)> = done.into_iter().map(|(n, s)| (s, n)).collect();
        ranked.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
        ranked.into_iter().map(|(_, name)| name).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn anchors(words: &[&str]) -> BTreeSet<String> {
        words.iter().map(|w| w.to_string()).collect()
    }

    #[test]
    fn a_word_ends_where_the_model_saw_words_end() {
        let model = LetterModel::train(["wald", "wald", "wale"]);
        let names = model.names("act", &anchors(&["wald"]), 8);
        assert!(names.contains(&"actwald".to_string()), "{names:?}");
    }

    #[test]
    fn the_anchors_pull_the_letters_toward_their_words() {
        let model = LetterModel::train(["forest", "forum", "motor", "moteur"]);
        let toward_motor = model.names("act", &anchors(&["motor"]), 8);
        let toward_forest = model.names("act", &anchors(&["forest"]), 8);
        assert_ne!(toward_motor, toward_forest);
        assert_eq!(toward_motor.first().map(String::as_str), Some("actmotor"));
    }

    #[test]
    fn the_same_input_gives_the_same_names() {
        let model = LetterModel::train(["alpha", "alter", "altar"]);
        let a = anchors(&["alpha"]);
        assert_eq!(model.names("act", &a, 9), model.names("act", &a, 9));
    }

    #[test]
    fn no_name_exceeds_the_maximum_length_or_the_minimum_suffix() {
        let model = LetterModel::train(["alpha", "alter", "altar"]);
        for name in model.names("act", &anchors(&["alpha"]), 7) {
            assert!(name.len() <= 7 && name.len() >= 3 + MIN_SUFFIX, "{name}");
        }
    }

    #[test]
    fn words_that_are_not_lowercase_ascii_or_too_short_are_ignored() {
        let model = LetterModel::train(["ab", "Été", "ok1"]);
        assert!(model.next.is_empty());
    }

    #[test]
    fn an_empty_model_proposes_nothing() {
        assert!(
            LetterModel::default()
                .names("act", &anchors(&["x"]), 8)
                .is_empty()
        );
    }
}
