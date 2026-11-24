use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

#[derive(Debug)]
pub struct SemanticKnowledgeBase {
    aliases: BTreeMap<String, BTreeSet<String>>,
    /// Concept → its broader concepts (a category: `animal` for `crocodile`).
    broader: BTreeMap<String, BTreeSet<String>>,
    /// Concept → concepts that accompany it in context.
    related: BTreeMap<String, BTreeSet<String>>,
}

/// Optional first line of a semantics file, so the file reads as a table.
pub const HEADER: &str = "concept\taliases\tbroader\trelated";

/// Weight of a word of the request's own concept, of its category, of its context.
pub const DIRECT: u8 = 3;
pub const BROADER: u8 = 2;
pub const RELATED: u8 = 1;

impl SemanticKnowledgeBase {
    pub fn from_tsv(source: &str) -> Result<Self, SemanticError> {
        let mut aliases: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let mut broader: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let mut related: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

        for (index, line) in source.lines().enumerate() {
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') || line == HEADER {
                continue;
            }

            // concept <TAB> aliases [<TAB> broader concepts [<TAB> related concepts]]
            let mut columns = line.splitn(4, '\t');
            let (Some(concept), Some(variants)) = (columns.next(), columns.next()) else {
                return Err(SemanticError::InvalidLine(index + 1));
            };
            let (broad, rel) = (columns.next(), columns.next());

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

            for (column, into) in [(broad, &mut broader), (rel, &mut related)] {
                let names = column
                    .into_iter()
                    .flat_map(|names| names.split('|'))
                    .map(|name| normalize(name.trim()))
                    .filter(|name| !name.is_empty());
                into.entry(concept.clone()).or_default().extend(names);
            }
        }

        Ok(Self {
            aliases,
            broader,
            related,
        })
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

    /// The concepts a request reaches, each with the weight of its closest
    /// path: its own (`DIRECT`), its category (`BROADER`), its context (`RELATED`).
    pub fn expand_weighted(&self, context: &str) -> BTreeMap<String, u8> {
        let mut weights: BTreeMap<String, u8> = BTreeMap::new();
        let mut raise = |concept: &String, weight: u8| {
            let slot = weights.entry(concept.clone()).or_insert(0);
            *slot = (*slot).max(weight);
        };

        for token in tokenize(context) {
            for concept in self.aliases.get(&token).into_iter().flatten() {
                raise(concept, DIRECT);
                for broad in self.broader.get(concept).into_iter().flatten() {
                    raise(broad, BROADER);
                }
                for rel in self.related.get(concept).into_iter().flatten() {
                    raise(rel, RELATED);
                }
            }
        }

        weights
    }

    /// The words of the weighted concepts, each with its best weight.
    pub fn weighted_words(&self, concepts: &BTreeMap<String, u8>) -> BTreeMap<String, u8> {
        self.aliases
            .iter()
            .filter_map(|(word, owners)| {
                let weight = owners.iter().filter_map(|c| concepts.get(c)).max()?;
                Some((word.clone(), *weight))
            })
            .collect()
    }

    /// Every word the base knows (concepts and aliases), sorted.
    pub fn terms(&self) -> impl Iterator<Item = &str> {
        self.aliases.keys().map(String::as_str)
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
    fn comments_and_optional_class_columns_are_accepted() {
        let source = "# a comment\ncroc\tcrocodile|krokodil\tanimal\tswamp|river\nanimal\tanimal|tier\nswamp\tswamp|sumpf";
        let kb = SemanticKnowledgeBase::from_tsv(source).unwrap_or_else(|e| panic!("{e}"));
        let weights = kb.expand_weighted("crocodile");
        assert_eq!(weights.get("croc"), Some(&DIRECT));
        assert_eq!(weights.get("animal"), Some(&BROADER));
        assert_eq!(weights.get("swamp"), Some(&RELATED));
        assert_eq!(weights.get("river"), Some(&RELATED));
    }

    #[test]
    fn a_header_line_is_skipped() {
        let kb = SemanticKnowledgeBase::from_tsv(&format!("{HEADER}\ncroc\tcrocodile\n"))
            .unwrap_or_else(|e| panic!("{e}"));
        assert!(kb.concepts_for("concept").is_empty());
        assert!(kb.concepts_for("crocodile").contains("croc"));
    }

    #[test]
    fn a_concept_reached_two_ways_keeps_its_closest_weight() {
        let kb = SemanticKnowledgeBase::from_tsv("a\txxx\tb\nb\tyyy\n")
            .unwrap_or_else(|e| panic!("{e}"));
        let weights = kb.expand_weighted("xxx yyy");
        assert_eq!(weights.get("b"), Some(&DIRECT));
    }

    #[test]
    fn the_words_of_a_class_carry_its_weight() {
        let kb = SemanticKnowledgeBase::from_tsv("croc\tcrocodile\tanimal\nanimal\tanimal|tier\n")
            .unwrap_or_else(|e| panic!("{e}"));
        let words = kb.weighted_words(&kb.expand_weighted("crocodile"));
        assert_eq!(words.get("crocodile"), Some(&DIRECT));
        assert_eq!(words.get("tier"), Some(&BROADER));
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
