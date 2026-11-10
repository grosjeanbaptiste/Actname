use actname::lexicon::{LexiconError, TextLexicon};
use actname::semantics::SemanticKnowledgeBase;
use actname::{Form, InMemoryLexicon, Lexicon, NamingCriteria, NamingService, Strategy};

fn criteria(context: &str) -> NamingCriteria {
    NamingCriteria {
        prefixes: vec!["act".into()],
        context: context.into(),
        max_length: 12,
        limit: 50,
        strategy: Strategy::Hybrid,
        exclusions: vec![],
    }
}

fn names<L: Lexicon>(service: &NamingService<'_, L>, context: &str) -> Vec<String> {
    let found = service
        .generate(&criteria(context))
        .unwrap_or_else(|error| panic!("{error}"));
    found.into_iter().map(|c| c.name).collect()
}

#[test]
fn text_lexicon_drives_the_names() {
    let text = "# suffix<TAB>form<TAB>concepts\nzork\tcomposed\tarchitecture|plans\n";
    let lexicon = TextLexicon::from_text(text).unwrap_or_else(|error| panic!("{error}"));
    let found = names(&NamingService::new(&lexicon), "architecture");
    assert!(found.contains(&"actzork".to_string()), "{found:?}");
    let default = names(&NamingService::new(&InMemoryLexicon), "architecture");
    assert!(default.contains(&"actplan".to_string()), "{default:?}");
    assert!(!found.contains(&"actplan".to_string()), "{found:?}");
}

#[test]
fn text_lexicon_keeps_form_and_concepts() {
    let lexicon = TextLexicon::from_text("zork\tcoined\tA|b\n").unwrap_or_else(|e| panic!("{e}"));
    let entry = &lexicon.entries()[0];
    assert_eq!((entry.suffix.as_str(), entry.form), ("zork", Form::Coined));
    assert_eq!(entry.concepts, ["A", "b"]);
}

#[test]
fn text_lexicon_errors_name_the_line() {
    let line = |text: &str| TextLexicon::from_text(text).err();
    assert_eq!(
        line("zork\tcomposed"),
        Some(LexiconError::Line(
            1,
            "expected suffix, form, concepts".into()
        ))
    );
    assert_eq!(
        line("ok\tcomposed\ta\nzork\tweird\ta"),
        Some(LexiconError::Line(2, "unknown form \"weird\"".into()))
    );
    assert_eq!(
        line("\tcomposed\ta"),
        Some(LexiconError::Line(1, "empty suffix".into()))
    );
    assert_eq!(
        line("zork\tcomposed\t|"),
        Some(LexiconError::Line(1, "no concept".into()))
    );
}

#[test]
fn embedded_lexicon_is_still_the_default() {
    let found = names(&NamingService::new(&InMemoryLexicon), "architecture");
    assert!(found.contains(&"actecture".to_string()));
}

#[test]
fn custom_knowledge_base_maps_new_words_to_concepts() {
    let base = SemanticKnowledgeBase::from_tsv("architecture\tzorp|zorps\n")
        .unwrap_or_else(|error| panic!("{error}"));
    let service = NamingService::new(&InMemoryLexicon).with_knowledge(base);
    assert!(names(&service, "zorp").contains(&"actecture".to_string()));
    let plain = NamingService::new(&InMemoryLexicon);
    assert!(
        plain
            .generate(&criteria("zorp"))
            .map(|c| c.is_empty())
            .unwrap_or(true)
    );
}
