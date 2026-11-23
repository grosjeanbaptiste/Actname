use actname::lexicon::TextLexicon;
use actname::semantics::SemanticKnowledgeBase;
use actname::{InMemoryLexicon, NamingCriteria, NamingService, Strategy};

/// A tiny LL-like base: concepts with their words in several languages.
const BASE: &str = "forest\tforest|wald|bois|foret|bosque\n\
                    engine\tengine|motor|moteur|motore\n\
                    house\thouse|haus|maison|huis|casa\n";

fn criteria(context: &str) -> NamingCriteria {
    NamingCriteria {
        prefixes: vec!["act".into()],
        context: context.into(),
        max_length: 10,
        limit: 12,
        strategy: Strategy::Statistical,
        exclusions: vec![],
    }
}

fn names(context: &str) -> Vec<String> {
    let knowledge = SemanticKnowledgeBase::from_tsv(BASE).unwrap_or_else(|e| panic!("{e}"));
    let service = NamingService::new(&InMemoryLexicon).with_knowledge(knowledge);
    service
        .generate(&criteria(context))
        .unwrap_or_else(|e| panic!("{e}"))
        .into_iter()
        .map(|c| c.name)
        .collect()
}

#[test]
fn the_same_request_always_gives_the_same_names() {
    assert_eq!(names("wald"), names("wald"));
    assert!(!names("wald").is_empty());
}

#[test]
fn every_name_starts_with_a_prefix_fits_the_length_and_is_lowercase_ascii() {
    for name in names("moteur") {
        assert!(name.starts_with("act"), "{name}");
        assert!(name.len() <= 10 && name.len() > 3, "{name}");
        assert!(name.chars().all(|c| c.is_ascii_lowercase()), "{name}");
    }
}

#[test]
fn the_request_steers_the_letters_toward_its_words() {
    // « wald » (de) ouvre le concept forest : ses mots donnent la suite de lettres.
    let forest = names("wald");
    assert!(
        forest.iter().any(|n| {
            let suffix = &n[3..];
            ["forest", "wald", "bois", "foret", "bosque"]
                .iter()
                .any(|w| suffix.len() >= 3 && w.contains(suffix))
        }),
        "{forest:?}"
    );
    assert_ne!(forest, names("moteur"));
}

#[test]
fn names_come_from_the_base_not_from_the_lexicon() {
    let lexicon =
        TextLexicon::from_text("zork\tcomposed\tforest\n").unwrap_or_else(|e| panic!("{e}"));
    let knowledge = SemanticKnowledgeBase::from_tsv(BASE).unwrap_or_else(|e| panic!("{e}"));
    let service = NamingService::new(&lexicon).with_knowledge(knowledge);
    let found = service
        .generate(&criteria("wald"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(found.iter().all(|c| c.name != "actzork"));
}

#[test]
fn a_request_with_no_known_word_gives_no_name() {
    assert!(names("zzzzz").is_empty());
}

#[test]
fn excluded_names_never_come_back() {
    let all = names("wald");
    let knowledge = SemanticKnowledgeBase::from_tsv(BASE).unwrap_or_else(|e| panic!("{e}"));
    let service = NamingService::new(&InMemoryLexicon).with_knowledge(knowledge);
    let mut c = criteria("wald");
    c.exclusions = vec![all[0].clone()];
    let again = service.generate(&c).unwrap_or_else(|e| panic!("{e}"));
    assert!(again.iter().all(|x| x.name != all[0]));
}

/// Same words, with the classes of the LL base: the category and the context of each sense.
const CLASSED: &str = "croc\tcrocodile|krokodil|crocodilo\tanimal\tswamp\n\
                       animal\tanimal|tier|animale\n\
                       swamp\tswamp|sumpf|marais\n";

fn classed(context: &str) -> Vec<String> {
    let knowledge = SemanticKnowledgeBase::from_tsv(CLASSED).unwrap_or_else(|e| panic!("{e}"));
    let service = NamingService::new(&InMemoryLexicon).with_knowledge(knowledge);
    let mut c = criteria(context);
    c.limit = 60;
    service
        .generate(&c)
        .unwrap_or_else(|e| panic!("{e}"))
        .into_iter()
        .map(|c| c.name)
        .collect()
}

#[test]
fn the_category_of_a_sense_also_steers_the_letters() {
    // « crocodile » ne contient pas « tier » ni « animal » : seule la catégorie les apporte.
    let found = classed("crocodile");
    assert!(
        found.iter().any(|n| {
            let suffix = &n[3..];
            suffix.len() >= 3
                && ["animal", "tier", "animale"]
                    .iter()
                    .any(|w| w.contains(suffix))
        }),
        "{found:?}"
    );
}

#[test]
fn the_context_of_a_sense_also_steers_the_letters() {
    let found = classed("crocodile");
    assert!(
        found.iter().any(|n| {
            let suffix = &n[3..];
            suffix.len() >= 3
                && ["swamp", "sumpf", "marais"]
                    .iter()
                    .any(|w| w.contains(suffix))
        }),
        "{found:?}"
    );
}

#[test]
fn the_request_word_ranks_above_its_category_and_its_context() {
    let found = classed("crocodile");
    let rank = |part: &str| found.iter().position(|n| n[3..].contains(part));
    assert!(rank("croc") < rank("tier"), "{found:?}");
    assert!(rank("tier") < rank("sump"), "{found:?}");
}
