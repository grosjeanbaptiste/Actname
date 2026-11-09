use actname::{InMemoryLexicon, NamingCriteria, NamingService, Strategy};

fn criteria(context: &str) -> NamingCriteria {
    NamingCriteria {
        prefixes: vec!["act".into(), "acte".into()],
        context: context.into(),
        max_length: 10,
        limit: 20,
        strategy: Strategy::Hybrid,
        exclusions: vec![],
    }
}

#[test]
fn rejects_missing_prefixes() {
    let mut input = criteria("architecture");
    input.prefixes.clear();

    assert!(input.validate().is_err());
}

#[test]
fn rejects_invalid_prefixes() {
    let mut input = criteria("architecture");
    input.prefixes = vec!["act-".into()];

    assert!(input.validate().is_err());
}

#[test]
fn rejects_empty_context() {
    let input = criteria(" ");

    assert!(input.validate().is_err());
}

#[test]
fn rejects_invalid_length_and_limit() {
    let mut input = criteria("architecture");
    input.max_length = 3;

    assert!(input.validate().is_err());

    input.max_length = 10;
    input.limit = 0;

    assert!(input.validate().is_err());
}

#[test]
fn generates_names_respecting_all_constraints() {
    let lexicon = InMemoryLexicon::default();
    let service = NamingService::new(&lexicon);

    let results = service
        .generate(&criteria(
            "conception de plans physiques bâtiments objets technologiques",
        ))
        .unwrap();

    assert!(!results.is_empty());

    assert!(results.iter().all(|candidate| {
        candidate.name.len() <= 10
            && candidate.name.starts_with("act")
            && candidate.name.chars().all(|c| c.is_ascii_lowercase())
    }));
}

#[test]
fn associates_architecture_with_relevant_names() {
    let lexicon = InMemoryLexicon::default();
    let service = NamingService::new(&lexicon);

    let results = service
        .generate(&criteria("architecture bâtiments plans conception"))
        .unwrap();

    assert!(
        results
            .iter()
            .any(|candidate| { candidate.name == "actecture" })
    );
}

#[test]
fn excludes_forbidden_names() {
    let lexicon = InMemoryLexicon::default();
    let service = NamingService::new(&lexicon);

    let mut input = criteria("architecture bâtiments plans");
    input.exclusions = vec!["actecture".into()];

    let results = service.generate(&input).unwrap();

    assert!(
        results
            .iter()
            .all(|candidate| { candidate.name != "actecture" })
    );
}

#[test]
fn respects_generation_strategy() {
    let lexicon = InMemoryLexicon::default();
    let service = NamingService::new(&lexicon);

    let mut input = criteria("architecture forme conception");

    input.strategy = Strategy::Composed;
    let composed = service.generate(&input).unwrap();

    assert!(
        composed
            .iter()
            .all(|candidate| { candidate.form == actname::Form::Composed })
    );

    input.strategy = Strategy::Coined;
    let coined = service.generate(&input).unwrap();

    assert!(
        coined
            .iter()
            .all(|candidate| { candidate.form == actname::Form::Coined })
    );
}

#[test]
fn produces_deterministic_unique_sorted_results() {
    let lexicon = InMemoryLexicon::default();
    let service = NamingService::new(&lexicon);
    let input = criteria("architecture conception plans");

    let first = service.generate(&input).unwrap();
    let second = service.generate(&input).unwrap();

    assert_eq!(first, second);

    let mut names: Vec<_> = first
        .iter()
        .map(|candidate| candidate.name.as_str())
        .collect();

    names.sort_unstable();
    names.dedup();

    assert_eq!(names.len(), first.len());

    assert!(
        first
            .windows(2)
            .all(|pair| { pair[0].score.total >= pair[1].score.total })
    );
}

#[test]
fn returns_no_names_without_semantic_evidence() {
    let lexicon = InMemoryLexicon::default();
    let service = NamingService::new(&lexicon);

    let results = service.generate(&criteria("xyzunrelatedword")).unwrap();

    assert!(results.is_empty());
}

#[test]
fn limits_number_of_results() {
    let lexicon = InMemoryLexicon::default();
    let service = NamingService::new(&lexicon);

    let mut input = criteria("conception plans bâtiments");
    input.limit = 3;

    let results = service.generate(&input).unwrap();

    assert!(results.len() <= 3);
}
