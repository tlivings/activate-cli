pub mod completions;
pub mod wrapper;

pub use completions::generate_completions;
pub use wrapper::{generate_wrapper, Shell};
