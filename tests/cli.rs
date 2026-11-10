use std::process::Command;

#[test]
fn cli_generates_names_from_parameters() {
    let output = Command::new(env!("CARGO_BIN_EXE_actname"))
        .args([
            "generate",
            "--prefix",
            "act",
            "--prefix",
            "acte",
            "--domain",
            "architecture bâtiments plans",
            "--max-length",
            "10",
            "--limit",
            "10",
        ])
        .output()
        .unwrap_or_else(|error| panic!("{error}"));

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("{error}"));

    assert!(stdout.contains("actecture"));
}

#[test]
fn cli_supports_json_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_actname"))
        .args([
            "generate",
            "--prefix",
            "act",
            "--domain",
            "architecture",
            "--max-length",
            "12",
            "--format",
            "json",
        ])
        .output()
        .unwrap_or_else(|error| panic!("{error}"));

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("{error}"));

    assert!(stdout.starts_with('['));
    assert!(stdout.contains("\"name\""));
    assert!(stdout.contains("\"score\""));
    assert!(stdout.contains("\"provenance\""));
    assert!(stdout.contains("\"name\":\"actetecture\""));
    assert!(stdout.contains("\"concepts\":[\"architecture\"]"));
    assert!(stdout.contains("\"linking_vowel:e\""));
}

#[test]
fn cli_rejects_unknown_options() {
    let output = Command::new(env!("CARGO_BIN_EXE_actname"))
        .args(["generate", "--unknown"])
        .output()
        .unwrap_or_else(|error| panic!("{error}"));

    assert!(!output.status.success());
}

#[test]
fn cli_displays_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_actname"))
        .arg("--help")
        .output()
        .unwrap_or_else(|error| panic!("{error}"));

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap_or_else(|error| panic!("{error}"));

    assert!(stdout.contains("generate"));
    assert!(stdout.contains("--domain"));
}

fn actname(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_actname"))
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("{error}"))
}

fn file(dir: &std::path::Path, name: &str, text: &str) -> String {
    let path = dir.join(name);
    std::fs::write(&path, text).unwrap_or_else(|error| panic!("{error}"));
    path.to_string_lossy().into_owned()
}

fn temp() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "actname-cli-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::create_dir_all(&dir).unwrap_or_else(|error| panic!("{error}"));
    dir
}

fn out(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn err(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn cli_lexicon_file_replaces_the_embedded_lexicon() {
    let lexicon = file(&temp(), "lex.tsv", "zork\tcomposed\tarchitecture\n");
    let output = actname(&[
        "generate",
        "--prefix",
        "act",
        "--domain",
        "architecture",
        "--lexicon",
        &lexicon,
    ]);
    assert!(output.status.success(), "{}", err(&output));
    assert!(out(&output).contains("actzork"));
}

#[test]
fn cli_bad_lexicon_names_the_file_and_the_line() {
    let lexicon = file(&temp(), "bad.tsv", "zork\tweird\ta\n");
    let output = actname(&[
        "generate",
        "--prefix",
        "act",
        "--domain",
        "architecture",
        "--lexicon",
        &lexicon,
    ]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        err(&output).contains("bad.tsv") && err(&output).contains("line 1"),
        "{}",
        err(&output)
    );
    let missing = actname(&[
        "generate",
        "--prefix",
        "act",
        "--domain",
        "x",
        "--lexicon",
        "/no/such.tsv",
    ]);
    assert!(err(&missing).contains("/no/such.tsv"));
}

#[test]
fn cli_scoring_file_sets_the_model() {
    let scoring = file(
        &temp(),
        "model.txt",
        "weight_semantic=0\nweight_readability=0\nweight_originality=1\noriginality_composed=33\n",
    );
    let output = actname(&[
        "generate",
        "--prefix",
        "act",
        "--domain",
        "architecture",
        "--strategy",
        "composed",
        "--format",
        "json",
        "--scoring",
        &scoring,
    ]);
    assert!(output.status.success(), "{}", err(&output));
    assert!(out(&output).contains("\"total\":33"), "{}", out(&output));
}

#[test]
fn cli_semantics_file_maps_new_words() {
    let semantics = file(&temp(), "sem.tsv", "architecture\tzorp\n");
    let output = actname(&[
        "generate",
        "--prefix",
        "act",
        "--domain",
        "zorp",
        "--semantics",
        &semantics,
    ]);
    assert!(output.status.success(), "{}", err(&output));
    assert!(out(&output).contains("actecture"));
}
