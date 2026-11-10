use std::fmt;

use crate::domain::{Form, Lexeme, Lexicon};

/// A lexicon read from text: one `suffix<TAB>composed|coined<TAB>concept|concept` per line,
/// `#` comments and blank lines ignored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextLexicon {
    entries: Vec<Lexeme>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexiconError {
    Line(usize, String),
}

impl fmt::Display for LexiconError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self::Line(line, why) = self;
        write!(f, "lexicon, line {line}: {why}")
    }
}

impl std::error::Error for LexiconError {}

impl TextLexicon {
    pub fn from_text(text: &str) -> Result<Self, LexiconError> {
        let mut entries = Vec::new();
        for (index, line) in text.lines().enumerate() {
            if line.trim().is_empty() || line.trim_start().starts_with('#') {
                continue;
            }
            let at = |why: &str| LexiconError::Line(index + 1, why.to_string());
            let columns: Vec<&str> = line.split('\t').collect();
            let [suffix, form, concepts] = columns[..] else {
                return Err(at("expected suffix, form, concepts"));
            };
            let form = match form.trim() {
                "composed" => Form::Composed,
                "coined" => Form::Coined,
                other => return Err(at(&format!("unknown form {other:?}"))),
            };
            let suffix = suffix.trim();
            if suffix.is_empty() {
                return Err(at("empty suffix"));
            }
            let concepts: Vec<String> = concepts
                .split('|')
                .map(str::trim)
                .filter(|c| !c.is_empty())
                .map(String::from)
                .collect();
            if concepts.is_empty() {
                return Err(at("no concept"));
            }
            entries.push(Lexeme {
                suffix: suffix.to_string(),
                concepts,
                form,
            });
        }
        Ok(Self { entries })
    }
}

impl Lexicon for TextLexicon {
    fn entries(&self) -> &[Lexeme] {
        &self.entries
    }
}
