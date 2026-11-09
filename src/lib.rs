pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod morphology;
pub mod semantics;

pub use application::NamingService;
pub use domain::{
    Candidate, CandidateProvenance, Form, Lexeme, Lexicon, NamingCriteria, NamingError, Score,
    Strategy,
};
pub use infrastructure::InMemoryLexicon;

/// Compiles the README examples so the documented API cannot drift from the code.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
