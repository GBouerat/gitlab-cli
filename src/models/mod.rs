//! Strongly-typed domain models for GitLab REST API entities.

pub mod ci;
pub mod issue;
pub mod mr;
pub mod project;
pub mod user;

pub use ci::{Job, Pipeline};
pub use issue::Issue;
pub use mr::MergeRequest;
pub use project::Project;
pub use user::User;
