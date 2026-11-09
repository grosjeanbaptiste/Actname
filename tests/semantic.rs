use actname::{InMemoryLexicon, NamingCriteria, NamingService, Strategy};

fn generate(context: &str) -> Vec<String> {
    let lexicon = InMemoryLexicon;
    let service = NamingService::new(&lexicon);

    let criteria = NamingCriteria {
        prefixes: vec!["act".into(), "acte".into()],
        context: context.into(),
        max_length: 10,
        limit: 100,
        strategy: Strategy::Hybrid,
        exclusions: vec![],
    };

    service
        .generate(&criteria)
        .unwrap_or_else(|error| panic!("{error}"))
        .into_iter()
        .map(|candidate| candidate.name)
        .collect()
}

#[test]
fn english_building_blueprints_match_architecture() {
    let results = generate("building blueprints");

    assert!(results.contains(&"actecture".to_string()));
}

#[test]
fn dutch_architecture_matches_building_concepts() {
    let results = generate("gebouwen bouwkunde ontwerp");

    assert!(results.contains(&"actecture".to_string()));
}

#[test]
fn spanish_architecture_matches_building_concepts() {
    let results = generate("edificios arquitectura planos");

    assert!(results.contains(&"actecture".to_string()));
}

#[test]
fn german_architecture_matches_building_concepts() {
    let results = generate("Gebäude Baupläne Architektur");

    assert!(results.contains(&"actecture".to_string()));
}

#[test]
fn english_mechanics_matches_kinematics() {
    let results = generate("mechanics motion machines");

    assert!(results.contains(&"actkinet".to_string()));
}

#[test]
fn english_legal_domain_matches_law() {
    let results = generate("legal statutes legislation");

    assert!(results.contains(&"actlex".to_string()));
}

#[test]
fn english_sports_domain_matches_tracking() {
    let results = generate("sports performance tracking");

    assert!(results.contains(&"actrack".to_string()));
}

#[test]
fn english_electronics_matches_circuits() {
    let results = generate("electronics circuits hardware");

    assert!(results.contains(&"actcircuit".to_string()));
}

#[test]
fn unknown_context_does_not_generate_unrelated_names() {
    let results = generate("qzxvplmn");

    assert!(results.is_empty());
}

#[test]
fn semantic_expansion_is_deterministic() {
    let first = generate("building blueprints architecture");
    let second = generate("building blueprints architecture");

    assert_eq!(first, second);
}

#[test]
fn semantic_expansion_preserves_existing_french_behavior() {
    let results = generate("conception de plans bâtiments architecture");

    assert!(results.contains(&"actecture".to_string()));
}

#[test]
fn semantic_expansion_does_not_match_partial_words() {
    let results = generate("architecturalxyz");

    assert!(results.is_empty());
}
