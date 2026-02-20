pub mod clone;
pub mod origin;
pub mod status;

pub use clone::{clone_repository, extract_repo_name};
pub use origin::detect_origin;
pub use status::GitStatus;
