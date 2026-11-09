use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

#[derive(Debug)]
pub struct SemanticKnowledgeBase {
    aliases: BTreeMap<String, BTreeSet<String>>,
}

impl SemanticKnowledgeBase {
    pub fn from_tsv(source: &str) -> Result<Self, SemanticError> {
        let mut aliases: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

        for (index, line) in source.lines().enumerate() {
            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            let Some((concept, variants)) = line.split_once('\t') else {
                return Err(SemanticError::InvalidLine(index + 1));
            };

            let concept = normalize(concept);

            if concept.is_empty() {
                return Err(SemanticError::InvalidLine(index + 1));
            }

            let mut terms = BTreeSet::new();
            terms.insert(concept.clone());

            for variant in variants.split('|') {
                let variant = normalize(variant.trim());

                if variant.is_empty() {
                    return Err(SemanticError::InvalidLine(index + 1));
                }

                terms.insert(variant);
            }

            for term in terms {
                aliases.entry(term).or_default().insert(concept.clone());
            }
        }

        Ok(Self { aliases })
    }

    pub fn expand(&self, context: &str) -> BTreeSet<String> {
        let mut expanded = BTreeSet::new();

        for token in tokenize(context) {
            expanded.insert(token.clone());

            if let Some(concepts) = self.aliases.get(&token) {
                expanded.extend(concepts.iter().cloned());
            }
        }

        expanded
    }

    pub fn concepts_for(&self, term: &str) -> BTreeSet<String> {
        let term = normalize(term);

        self.aliases.get(&term).cloned().unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticError {
    InvalidLine(usize),
}

impl std::fmt::Display for SemanticError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLine(line) => {
                write!(formatter, "invalid semantic knowledge base line {line}")
            }
        }
    }
}

impl std::error::Error for SemanticError {}

pub fn normalize(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .map(|character| match character {
            'à' | 'â' | 'ä' | 'á' | 'ã' | 'å' => 'a',
            'ç' => 'c',
            'è' | 'é' | 'ê' | 'ë' => 'e',
            'ì' | 'í' | 'î' | 'ï' => 'i',
            'ñ' => 'n',
            'ò' | 'ó' | 'ô' | 'ö' | 'õ' => 'o',
            'ù' | 'ú' | 'û' | 'ü' => 'u',
            'ý' | 'ÿ' => 'y',
            'ß' => 's',
            other => other,
        })
        .collect()
}

pub fn tokenize(value: &str) -> BTreeSet<String> {
    let normalized = normalize(value);

    normalized
        .split(|character: char| !character.is_ascii_alphabetic())
        .filter(|token| token.len() > 2)
        .map(str::to_owned)
        .collect()
}

pub fn default_knowledge_base() -> &'static SemanticKnowledgeBase {
    static INSTANCE: OnceLock<SemanticKnowledgeBase> = OnceLock::new();

    INSTANCE.get_or_init(|| {
        SemanticKnowledgeBase::from_tsv(include_str!("../resources/semantic.tsv"))
            .unwrap_or_else(|error| panic!("invalid embedded semantic knowledge base: {error}"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_semantic_aliases() {
        let source = "architecture\tarchitecture|building|gebouw";
        let knowledge_base =
            SemanticKnowledgeBase::from_tsv(source).unwrap_or_else(|error| panic!("{error}"));

        assert!(
            knowledge_base
                .concepts_for("building")
                .contains("architecture")
        );
    }

    #[test]
    fn rejects_malformed_lines() {
        let result = SemanticKnowledgeBase::from_tsv("invalid");

        assert_eq!(
            result
                .err()
                .unwrap_or_else(|| panic!("expected operation to fail")),
            SemanticError::InvalidLine(1)
        );
    }

    #[test]
    fn rejects_empty_aliases() {
        let result = SemanticKnowledgeBase::from_tsv("architecture\tbuilding|");

        assert_eq!(
            result
                .err()
                .unwrap_or_else(|| panic!("expected operation to fail")),
            SemanticError::InvalidLine(1)
        );
    }

    #[test]
    fn normalizes_accents() {
        assert_eq!(
            normalize("Bâtiments Électroniques"),
            "batiments electroniques"
        );
    }

    #[test]
    fn tokenizes_without_partial_matches() {
        let tokens = tokenize("architecture, bâtiments et objets");

        assert!(tokens.contains("architecture"));
        assert!(tokens.contains("batiments"));
        assert!(!tokens.contains("architect"));
    }

    #[test]
    fn expands_multiple_languages() {
        let knowledge_base = default_knowledge_base();

        let english = knowledge_base.expand("building");
        let dutch = knowledge_base.expand("gebouwen");
        let spanish = knowledge_base.expand("edificios");

        assert!(english.contains("batiments"));
        assert!(dutch.contains("batiments"));
        assert!(spanish.contains("batiments"));
    }

    #[test]
    fn preserves_unknown_tokens() {
        let expanded = default_knowledge_base().expand("unknownconcept");

        assert!(expanded.contains("unknownconcept"));
        assert_eq!(expanded.len(), 1);
    }
}
