//! Strongly-typed domain models for GitLab REST API entities.

pub mod branch;
pub mod ci;
pub mod deployment;
pub mod environment;
pub mod epic;
pub mod group;
pub mod issue;
pub mod milestone;
pub mod mr;
pub mod note;
pub mod package;
pub mod project;
pub mod registry;
pub mod release;
pub mod snippet;
pub mod tag;
pub mod user;
pub mod variable;

pub use branch::Branch;
pub use ci::{Job, Pipeline};
pub use deployment::Deployment;
pub use environment::Environment;
pub use epic::Epic;
pub use group::{Group, GroupMember};
pub use issue::Issue;
pub use milestone::Milestone;
pub use mr::MergeRequest;
pub use note::{MrApprovals, Note};
pub use package::{Package, PackageFile};
pub use project::Project;
pub use registry::{RegistryRepository, RegistryTag};
pub use release::Release;
pub use snippet::Snippet;
pub use tag::Tag;
pub use user::User;
pub use variable::CiVariable;
