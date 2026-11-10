//! `actname generate` takes its context from the repository's README when `--domain` is absent.
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

/// A fresh empty directory, removed when dropped.
struct Repo(PathBuf);

impl Repo {
    fn new(label: &str) -> Self {
        let dir =
            std::env::temp_dir().join(format!("actname-readme-{}-{label}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("{e}"));
        Repo(dir)
    }

    fn write(&self, name: &str, text: &str) {
        fs::write(self.0.join(name), text).unwrap_or_else(|e| panic!("{e}"));
    }

    fn actname(&self, extra: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_actname"))
            .args([
                "generate",
                "--prefix",
                "act",
                "--max-length",
                "12",
                "--limit",
                "20",
            ])
            .args(extra)
            .current_dir(&self.0)
            .output()
            .unwrap_or_else(|e| panic!("{e}"))
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

#[test]
fn the_readme_of_the_repository_is_the_context_when_there_is_no_domain() {
    let repo = Repo::new("default");
    repo.write(
        "README.md",
        "# Plans\n\nArchitecture bâtiments plans conception\n",
    );
    let o = repo.actname(&[]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(out(&o).contains("actecture"), "{}", out(&o));
}

#[test]
fn a_blank_readme_gives_no_context_and_the_usual_error() {
    let repo = Repo::new("blank");
    repo.write("README.md", "  \n\n");
    let o = repo.actname(&[]);
    assert!(!o.status.success());
    assert!(
        err(&o).contains("semantic context is required"),
        "{}",
        err(&o)
    );
}

#[test]
fn no_readme_and_no_domain_says_what_to_do() {
    let repo = Repo::new("missing");
    let o = repo.actname(&[]);
    assert!(!o.status.success());
    let e = err(&o);
    assert!(e.contains("semantic context is required"), "{e}");
    assert!(e.contains("--domain") && e.contains("README.md"), "{e}");
}

#[test]
fn an_explicit_domain_beats_the_readme() {
    let repo = Repo::new("domain-wins");
    repo.write("README.md", "zzzz qqqq xxxx");
    let o = repo.actname(&["--domain", "architecture"]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(out(&o).contains("act"), "{}", out(&o));
}

#[test]
fn readme_option_points_to_another_file() {
    let repo = Repo::new("other-file");
    repo.write("NOTES.md", "architecture bâtiments plans");
    let o = repo.actname(&["--readme", "NOTES.md"]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(out(&o).contains("actecture"), "{}", out(&o));
}

#[test]
fn an_unreadable_readme_is_an_error_naming_it_not_a_missing_one() {
    let repo = Repo::new("unreadable");
    fs::create_dir(repo.0.join("README.md")).unwrap_or_else(|e| panic!("{e}"));
    let o = repo.actname(&[]);
    assert!(!o.status.success());
    assert!(err(&o).contains("README.md"), "{}", err(&o));
    assert!(
        !err(&o).contains("semantic context is required"),
        "{}",
        err(&o)
    );
}

#[test]
fn a_readme_option_naming_a_missing_file_is_an_error_not_a_silent_fallback() {
    let repo = Repo::new("explicit-missing");
    repo.write("README.md", "architecture bâtiments plans");
    let o = repo.actname(&["--readme", "NOPE.md"]);
    assert!(!o.status.success());
    assert!(err(&o).contains("NOPE.md"), "{}", err(&o));
}
