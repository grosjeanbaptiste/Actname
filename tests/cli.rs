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
