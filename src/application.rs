use std::collections::BTreeSet;

use crate::domain::{
    Candidate, CandidateProvenance, Form, Lexicon, NamingCriteria, NamingError, Strategy,
    evaluate_matches_with, evaluate_with, semantic_tokens,
};
use crate::letters::LetterModel;
use crate::morphology::{
    MorphologicalGenerator, MorphologicalRequest, MorphologicalStrategy, morphological_quality,
};
use crate::scoring::ScoringModel;
use crate::semantics::SemanticKnowledgeBase;

/// Whether `suffix` and `word` have a run of three letters in common.
fn shares_three_letters(suffix: &str, word: &str) -> bool {
    suffix
        .as_bytes()
        .windows(3)
        .any(|run| word.as_bytes().windows(3).any(|other| other == run))
}

pub struct NamingService<'a, L: Lexicon> {
    lexicon: &'a L,
    model: ScoringModel,
    knowledge: Option<SemanticKnowledgeBase>,
}

impl<'a, L: Lexicon> NamingService<'a, L> {
    pub fn new(lexicon: &'a L) -> Self {
        Self {
            lexicon,
            model: ScoringModel::default(),
            knowledge: None,
        }
    }

    /// Words are expanded to concepts with this base instead of the embedded one.
    pub fn with_knowledge(mut self, knowledge: SemanticKnowledgeBase) -> Self {
        self.knowledge = Some(knowledge);
        self
    }

    pub fn with_model(mut self, model: ScoringModel) -> Self {
        self.model = model;
        self
    }

    pub fn generate(&self, criteria: &NamingCriteria) -> Result<Vec<Candidate>, NamingError> {
        criteria.validate()?;
        self.model.validate().map_err(NamingError::Invalid)?;

        let context = match &self.knowledge {
            Some(knowledge) => knowledge.expand(&criteria.context),
            None => semantic_tokens(&criteria.context),
        };

        let exclusions: BTreeSet<String> = criteria
            .exclusions
            .iter()
            .map(|name| name.to_ascii_lowercase())
            .collect();

        let mut candidates = if criteria.strategy == Strategy::Statistical {
            self.statistical_candidates(criteria, &exclusions)
        } else {
            let mut found = self.lexical_candidates(criteria, &context, &exclusions);
            found.extend(self.morphological_candidates(criteria, &context));
            found
        };

        let statistical = criteria.strategy == Strategy::Statistical;
        candidates.sort_by(|left, right| {
            let by_score = right
                .score
                .total
                .cmp(&left.score.total)
                .then_with(|| right.score.semantic.cmp(&left.score.semantic));
            // The letter model's own order breaks ties (the sort is stable).
            if statistical {
                by_score
            } else {
                by_score.then_with(|| left.name.cmp(&right.name))
            }
        });

        let mut seen = BTreeSet::new();

        candidates.retain(|candidate| seen.insert(candidate.name.clone()));

        if statistical {
            // `actile`, `actilec`, `actiler`… are one name: keep the best of a family.
            let mut kept: Vec<String> = Vec::new();
            candidates.retain(|candidate| {
                let related = kept.iter().any(|name| {
                    candidate.name.starts_with(name) || name.starts_with(&candidate.name)
                });
                if !related {
                    kept.push(candidate.name.clone());
                }
                !related
            });
        }

        candidates.truncate(criteria.limit);

        Ok(candidates)
    }

    /// Names drawn letter by letter from a model trained on every word of the
    /// semantic base, steered by the words of the request's concepts.
    fn statistical_candidates(
        &self,
        criteria: &NamingCriteria,
        exclusions: &BTreeSet<String>,
    ) -> Vec<Candidate> {
        let knowledge = self
            .knowledge
            .as_ref()
            .unwrap_or_else(|| crate::semantics::default_knowledge_base());
        let weights = knowledge.expand_weighted(&criteria.context);
        let anchors = knowledge.weighted_words(&weights);
        if anchors.is_empty() {
            return Vec::new();
        }
        let model = LetterModel::train(knowledge.terms());
        let mut candidates = Vec::new();
        for prefix in &criteria.prefixes {
            for name in model.names(prefix, &anchors, criteria.max_length) {
                if exclusions.contains(&name) {
                    continue;
                }
                let suffix = &name[prefix.len()..];
                let concepts: BTreeSet<String> = anchors
                    .keys()
                    .filter(|word| shares_three_letters(suffix, word))
                    .flat_map(|word| knowledge.concepts_for(word))
                    .filter(|concept| weights.contains_key(concept))
                    .collect();
                // A name rooted in the request's own words scores like three concepts,
                // one rooted in its category like two, in its context like one.
                let closest = concepts
                    .iter()
                    .filter_map(|concept| weights.get(concept))
                    .max()
                    .map_or(0, |weight| usize::from(*weight));
                if let Some(score) =
                    evaluate_matches_with(&self.model, &name, Form::Coined, closest)
                {
                    candidates.push(Candidate {
                        name,
                        form: Form::Coined,
                        score,
                        provenance: CandidateProvenance {
                            concepts: concepts.into_iter().collect(),
                            transformations: vec!["letter_model:order2".into()],
                        },
                    });
                }
            }
        }
        candidates
    }

    fn lexical_candidates(
        &self,
        criteria: &NamingCriteria,
        context: &BTreeSet<String>,
        exclusions: &BTreeSet<String>,
    ) -> Vec<Candidate> {
        let mut candidates = Vec::new();

        for lexeme in self.lexicon.entries() {
            if !criteria.strategy.accepts(lexeme.form) {
                continue;
            }

            for prefix in &criteria.prefixes {
                if prefix.ends_with('e') && lexeme.suffix.starts_with('e') {
                    continue;
                }

                let (name, fused) = crate::morphology::fuse(prefix, &lexeme.suffix);

                if name.len() > criteria.max_length
                    || !name.chars().all(|character| character.is_ascii_lowercase())
                    || exclusions.contains(&name)
                {
                    continue;
                }

                if let Some(score) = evaluate_with(&self.model, &name, lexeme, context) {
                    let concepts = lexeme
                        .concepts
                        .iter()
                        .filter(|concept| context.contains(&crate::domain::normalize(concept)))
                        .map(|concept| crate::domain::normalize(concept))
                        .collect::<BTreeSet<_>>()
                        .into_iter()
                        .collect();
                    let transformations = if fused {
                        vec!["boundary_deduplication".into()]
                    } else {
                        vec![
                            "prefix_suffix_composition".into(),
                            format!("lexical_suffix:{}", lexeme.suffix),
                        ]
                    };

                    candidates.push(Candidate {
                        name,
                        form: lexeme.form,
                        score,
                        provenance: CandidateProvenance {
                            concepts,
                            transformations,
                        },
                    });
                }
            }
        }

        candidates
    }

    fn morphological_candidates(
        &self,
        criteria: &NamingCriteria,
        context: &BTreeSet<String>,
    ) -> Vec<Candidate> {
        let roots: Vec<String> = context
            .iter()
            .filter(|concept| {
                self.lexicon.entries().iter().any(|lexeme| {
                    lexeme
                        .concepts
                        .iter()
                        .any(|known| crate::domain::normalize(known) == **concept)
                })
            })
            .cloned()
            .collect();

        if roots.is_empty() {
            return Vec::new();
        }

        let strategy = match criteria.strategy {
            Strategy::Hybrid => MorphologicalStrategy::Hybrid,
            Strategy::Composed => MorphologicalStrategy::Composed,
            Strategy::Coined => MorphologicalStrategy::Coined,
            Strategy::Statistical => return Vec::new(),
        };

        let request = MorphologicalRequest {
            roots,
            prefixes: criteria.prefixes.clone(),
            max_length: criteria.max_length,
            limit: usize::MAX,
            strategy,
            exclusions: criteria.exclusions.clone(),
        };

        let generator = MorphologicalGenerator;

        generator
            .generate(&request)
            .into_iter()
            .filter_map(|candidate| {
                let form = match candidate.strategy {
                    MorphologicalStrategy::Composed => Form::Composed,
                    MorphologicalStrategy::Coined => Form::Coined,
                    MorphologicalStrategy::Hybrid => return None,
                };

                let matches = self
                    .lexicon
                    .entries()
                    .iter()
                    .filter(|lexeme| {
                        lexeme
                            .concepts
                            .iter()
                            .any(|concept| crate::domain::normalize(concept) == candidate.root)
                    })
                    .flat_map(|lexeme| lexeme.concepts.iter())
                    .filter(|concept| context.contains(&crate::domain::normalize(concept)))
                    .map(|concept| crate::domain::normalize(concept))
                    .collect::<BTreeSet<_>>()
                    .len();

                evaluate_matches_with(&self.model, &candidate.name, form, matches).map(
                    |mut score| {
                        let quality = morphological_quality(&candidate);

                        if form == Form::Coined {
                            score.originality = self.blend_originality(score.originality, quality);
                            score.recalculate_total(&self.model.weights);
                        }

                        Candidate {
                            name: candidate.name,
                            form,
                            score,
                            provenance: CandidateProvenance {
                                concepts: vec![candidate.root],
                                transformations: candidate.transformations,
                            },
                        }
                    },
                )
            })
            .collect()
    }

    fn blend_originality(&self, form_value: u8, quality: u8) -> u8 {
        let (wf, wq) = (
            u32::from(self.model.coined_blend_form),
            u32::from(self.model.coined_blend_quality),
        );
        let sum = (wf + wq).max(1);
        ((u32::from(form_value) * wf + u32::from(quality) * wq + sum / 2) / sum) as u8
    }
}
