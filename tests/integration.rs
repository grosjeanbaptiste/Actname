use actname::{Form, InMemoryLexicon, NamingCriteria, NamingService, Strategy};

fn criteria(context: &str) -> NamingCriteria {
    NamingCriteria {
        prefixes: vec!["act".into(), "acte".into()],
        context: context.into(),
        max_length: 12,
        limit: 100,
        strategy: Strategy::Hybrid,
        exclusions: vec![],
    }
}

fn generate(context: &str) -> Vec<actname::Candidate> {
    let lexicon = InMemoryLexicon;
    let service = NamingService::new(&lexicon);

    service
        .generate(&criteria(context))
        .unwrap_or_else(|error| panic!("{error}"))
}

#[test]
fn integrates_morphological_candidates() {
    let results = generate("architecture");

    assert!(
        results
            .iter()
            .any(|candidate| { candidate.name == "actarchi" && candidate.form == Form::Coined })
    );
}

#[test]
fn retains_existing_lexical_candidates() {
    let results = generate("architecture bâtiments plans conception");

    assert!(
        results
            .iter()
            .any(|candidate| { candidate.name == "actecture" })
    );

    assert!(
        results
            .iter()
            .any(|candidate| { candidate.name == "acteplan" })
    );
}

#[test]
fn rejects_unrelated_context() {
    assert!(generate("xyzunrelatedword").is_empty());
}

#[test]
fn supports_multilingual_semantics() {
    let results = generate("building blueprints");

    assert!(results.iter().any(|candidate| {
        candidate.name == "actbatim" || candidate.name == "actplans" || candidate.name == "actplan"
    }));
}

#[test]
fn respects_coined_strategy() {
    let lexicon = InMemoryLexicon;
    let service = NamingService::new(&lexicon);

    let mut input = criteria("architecture");
    input.strategy = Strategy::Coined;

    let results = service
        .generate(&input)
        .unwrap_or_else(|error| panic!("{error}"));

    assert!(
        results
            .iter()
            .any(|candidate| { candidate.name == "actarchi" })
    );

    assert!(
        results
            .iter()
            .all(|candidate| { candidate.form == Form::Coined })
    );
}

#[test]
fn respects_composed_strategy() {
    let lexicon = InMemoryLexicon;
    let service = NamingService::new(&lexicon);

    let mut input = criteria("architecture");
    input.strategy = Strategy::Composed;

    let results = service
        .generate(&input)
        .unwrap_or_else(|error| panic!("{error}"));

    assert!(
        results
            .iter()
            .all(|candidate| { candidate.form == Form::Composed })
    );

    assert!(
        !results
            .iter()
            .any(|candidate| { candidate.name == "actarchi" })
    );
}

#[test]
fn applies_exclusions_to_all_sources() {
    let lexicon = InMemoryLexicon;
    let service = NamingService::new(&lexicon);

    let mut input = criteria("architecture");
    input.exclusions = vec!["actarchi".into(), "actecture".into()];

    let results = service
        .generate(&input)
        .unwrap_or_else(|error| panic!("{error}"));

    assert!(
        results
            .iter()
            .all(|candidate| { candidate.name != "actarchi" && candidate.name != "actecture" })
    );
}

#[test]
fn preserves_deterministic_ranking() {
    let first = generate("architecture bâtiments conception");
    let second = generate("architecture bâtiments conception");

    assert_eq!(first, second);

    assert!(
        first
            .windows(2)
            .all(|pair| { pair[0].score.total >= pair[1].score.total })
    );

    let names: std::collections::BTreeSet<_> =
        first.iter().map(|candidate| &candidate.name).collect();

    assert_eq!(names.len(), first.len());
}

#[test]
fn enforces_maximum_length() {
    let results = generate("architecture bâtiments conception");

    assert!(
        results
            .iter()
            .all(|candidate| { candidate.name.len() <= 12 })
    );
}

#[test]
fn preserves_semantic_score_consistency() {
    let results = generate("architecture");

    assert!(
        results
            .iter()
            .all(|candidate| { candidate.score.semantic > 0 && candidate.score.total <= 100 })
    );
}
