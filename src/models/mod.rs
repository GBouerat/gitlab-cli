//! Strongly-typed domain models for GitLab REST API entities.

pub mod branch;
pub mod ci;
pub mod group;
pub mod issue;
pub mod mr;
pub mod project;
pub mod release;
pub mod snippet;
pub mod user;
pub mod variable;

pub use branch::Branch;
pub use ci::{Job, Pipeline};
pub use group::{Group, GroupMember};
pub use issue::Issue;
pub use mr::MergeRequest;
pub use project::Project;
pub use release::Release;
pub use snippet::Snippet;
pub use user::User;
pub use variable::CiVariable;
