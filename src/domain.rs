use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    Hybrid,
    Composed,
    Coined,
}

impl Strategy {
    pub fn accepts(self, form: Form) -> bool {
        matches!(
            (self, form),
            (Self::Hybrid, _) | (Self::Composed, Form::Composed) | (Self::Coined, Form::Coined)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Form {
    Composed,
    Coined,
}

impl Form {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Composed => "composed",
            Self::Coined => "coined",
        }
    }
}

#[derive(Debug, Clone)]
pub struct NamingCriteria {
    pub prefixes: Vec<String>,
    pub context: String,
    pub max_length: usize,
    pub limit: usize,
    pub strategy: Strategy,
    pub exclusions: Vec<String>,
}

impl NamingCriteria {
    pub fn validate(&self) -> Result<(), NamingError> {
        if self.prefixes.is_empty() {
            return Err(NamingError::Invalid("at least one prefix is required"));
        }

        if self.prefixes.iter().any(|prefix| {
            prefix.is_empty()
                || !prefix
                    .chars()
                    .all(|character| character.is_ascii_lowercase())
        }) {
            return Err(NamingError::Invalid(
                "prefixes must contain lowercase ASCII letters only",
            ));
        }

        if self.context.trim().is_empty() {
            return Err(NamingError::Invalid("semantic context is required"));
        }

        if self.max_length == 0
            || self
                .prefixes
                .iter()
                .all(|prefix| prefix.len() >= self.max_length)
        {
            return Err(NamingError::Invalid(
                "maximum length must exceed at least one prefix length",
            ));
        }

        if self.limit == 0 {
            return Err(NamingError::Invalid("result limit must be positive"));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NamingError {
    Invalid(&'static str),
}

impl fmt::Display for NamingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(message) => write!(formatter, "{message}"),
        }
    }
}

impl Error for NamingError {}

#[derive(Debug, Clone, Copy)]
pub struct Lexeme {
    pub suffix: &'static str,
    pub concepts: &'static [&'static str],
    pub form: Form,
}

pub trait Lexicon {
    fn entries(&self) -> &[Lexeme];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Score {
    pub semantic: u8,
    pub readability: u8,
    pub originality: u8,
    pub total: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub name: String,
    pub form: Form,
    pub score: Score,
}

pub fn normalize(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .map(|character| match character {
            'à' | 'â' | 'ä' | 'á' => 'a',
            'ç' => 'c',
            'è' | 'é' | 'ê' | 'ë' => 'e',
            'ì' | 'í' | 'î' | 'ï' => 'i',
            'ñ' => 'n',
            'ò' | 'ó' | 'ô' | 'ö' => 'o',
            'ù' | 'ú' | 'û' | 'ü' => 'u',
            'ý' | 'ÿ' => 'y',
            other => other,
        })
        .collect()
}

pub fn semantic_tokens(context: &str) -> BTreeSet<String> {
    crate::semantics::default_knowledge_base().expand(context)
}

pub fn evaluate(name: &str, lexeme: &Lexeme, context: &BTreeSet<String>) -> Option<Score> {
    let matches = lexeme
        .concepts
        .iter()
        .filter(|concept| context.contains(&normalize(concept)))
        .count();

    evaluate_matches(name, lexeme.form, matches)
}

pub fn evaluate_matches(name: &str, form: Form, matches: usize) -> Option<Score> {
    if matches == 0 {
        return None;
    }

    let semantic = (40 + (matches as u64).saturating_mul(12)).min(100) as u8;

    let length_penalty = name.len().saturating_sub(7) as i32 * 3;

    let consonant_penalty = name
        .as_bytes()
        .windows(3)
        .filter(|window| {
            window
                .iter()
                .all(|byte| !matches!(*byte, b'a' | b'e' | b'i' | b'o' | b'u' | b'y'))
        })
        .count() as i32
        * 7;

    let repetition_penalty = name
        .as_bytes()
        .windows(2)
        .filter(|window| window[0] == window[1])
        .count() as i32
        * 6;

    let readability =
        (100 - length_penalty - consonant_penalty - repetition_penalty).clamp(0, 100) as u8;

    let originality = match form {
        Form::Composed => 70,
        Form::Coined => 90,
    };

    let weighted =
        u16::from(semantic) * 45 + u16::from(readability) * 30 + u16::from(originality) * 25;

    let total = ((weighted + 50) / 100) as u8;

    Some(Score {
        semantic,
        readability,
        originality,
        total,
    })
}
