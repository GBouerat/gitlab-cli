//! GitLab REST API client module.
//!
//! Provides the primary [`GitLabClient`] as well as error and pagination types.

pub mod client;
pub mod error;
pub mod pagination;

pub use client::{CreateMrParams, GitLabClient, USER_AGENT_STRING};
pub use error::ApiError;
pub use pagination::PaginationInfo;
