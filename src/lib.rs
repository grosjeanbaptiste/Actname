pub mod application;
pub mod domain;
pub mod infrastructure;

pub use application::NamingService;
pub use domain::{Candidate, Form, Lexeme, Lexicon, NamingCriteria, NamingError, Score, Strategy};
pub use infrastructure::InMemoryLexicon;
