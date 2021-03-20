use anyhow::*;
use serde::{Deserialize, Serialize};

use crate::git::Repository;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepoVersion {
    pub repo: String,
    pub version: Version,
}

impl Default for RepoVersion {
    fn default() -> Self {
        Version::default().into()
    }
}

impl From<Version> for RepoVersion {
    fn from(version: Version) -> Self {
        Self {
            version,
            repo: "https://github.com/endless-sky/endless-sky".to_owned(),
        }
    }
}

impl RepoVersion {
    pub fn checkout(&self, repo: &Repository) -> Result<()> {
        self.version.checkout(repo)
    }

    pub fn is_valid(&self) -> bool {
        !self.repo.is_empty() && self.version.is_valid()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Version {
    Commit(String),
    BranchHead(String),
    Tag(String),
}

impl Default for Version {
    fn default() -> Self {
        Self::BranchHead("master".to_owned())
    }
}

impl Version {
    fn checkout(&self, repo: &Repository) -> Result<()> {
        match self {
            Version::Commit(r) | Version::Tag(r) | Version::BranchHead(r) => repo.checkout(r)?,
        }
        Ok(())
    }

    fn is_valid(&self) -> bool {
        match self {
            Version::Commit(s) => {
                !s.is_empty() && s.chars().all(|x| x.is_numeric() || x.is_ascii_alphabetic())
            }
            Version::BranchHead(s) => !s.is_empty(),
            Version::Tag(s) => !s.is_empty(),
        }
    }
}
