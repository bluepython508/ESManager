use anyhow::*;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub struct Repository(PathBuf);

impl Repository {
    pub  fn clone(url: impl AsRef<str>, location: impl AsRef<Path>) -> Result<Self> {
        let success = Command::new("git")
            .arg("clone")
            .arg(url.as_ref())
            .arg(location.as_ref())
            .spawn()?
            .wait()?
            .success();
        if !success {
            bail!("Failed to clone repo");
        }
        Ok(Self(location.as_ref().to_owned()))
    }

    pub fn open(location: impl AsRef<Path>) -> Result<Self> {
        let location = location.as_ref().to_owned();
        if !location.exists() {
            bail!("Repo must exist");
        }
        Ok(Self(location))
    }

    pub fn checkout(&self, spec: impl AsRef<str>) -> Result<()> {
        let success = Command::new("git")
            .current_dir(&self.0)
            .arg("checkout")
            .arg(spec.as_ref())
            .spawn()?
            .wait()?
            .success();
        if !success {
            bail!("Checking out {} failed!", spec.as_ref());
        }
        Ok(())
    }
}
