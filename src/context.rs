//! Where the semantic context of a name comes from when `--domain` is not given.

/// The README text as a context, or `None` when it holds nothing but whitespace.
pub fn from_readme(text: &str) -> Option<String> {
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_readme_with_text_is_the_context_trimmed() {
        assert_eq!(
            from_readme("\n  Plans d'architecture\n\n"),
            Some("Plans d'architecture".to_string())
        );
    }

    #[test]
    fn a_blank_readme_is_no_context() {
        for blank in ["", "   ", "\n\n", "\t \r\n"] {
            assert_eq!(from_readme(blank), None, "{blank:?}");
        }
    }
}
