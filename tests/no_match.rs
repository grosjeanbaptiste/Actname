//! A context the data knows nothing about is an error that says so, never an empty answer.
use std::process::Command;

#[test]
fn a_context_matching_no_concept_is_an_error_naming_the_context() {
    let output = Command::new(env!("CARGO_BIN_EXE_actname"))
        .args([
            "generate",
            "--prefix",
            "act",
            "--domain",
            "recettes de cuisine",
        ])
        .output()
        .unwrap_or_else(|e| panic!("{e}"));
    let err = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "{err}");
    assert!(err.contains("no name matches"), "{err}");
    assert!(err.contains("--semantics"), "{err}");
}
