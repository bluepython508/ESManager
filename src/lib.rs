use std::path::PathBuf;

use anyhow::*;
use git::Repository;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use version::RepoVersion;

mod git;
pub mod version;

#[cfg(target_os = "linux")]
mod os {
    use anyhow::*;
    use serde::{Deserialize, Serialize};
    use std::{
        path::{Path, PathBuf},
        process::Command,
    };

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Binary {
        path: PathBuf,
    }

    impl Binary {
        pub fn compile(mut location: PathBuf) -> Result<Self> {
            let code = Command::new("scons")
                .current_dir(&location)
                .spawn()?
                .wait()?;
            if !code.success() {
                bail!("Compilation failed!");
            }
            location.push("endless-sky");
            if !location.exists() {
                bail!("Binary is missing!");
            }
            Ok(Self { path: location })
        }

        pub fn command(&self) -> Command {
            Command::new(&self.path)
        }

        pub fn file(&self) -> &Path {
            &self.path
        }
    }
}
#[cfg(target_os = "macos")]
mod os {
    use anyhow::*;
    use serde::{Deserialize, Serialize};
    use std::{path::PathBuf, process::Command};

    #[derive(Debug, Serialize, Clone, Deserialize)]
    pub struct Binary {
        path: PathBuf,
    }

    impl Binary {
        pub fn compile(mut location: PathBuf) -> Result<Self> {
            let code = Command::new("xcodebuild")
                .current_dir(&location)
                .spawn()?
                .wait()?;
            if !code.success() {
                bail!("Compilation failed!");
            }
            location.push("build");
            location.push("Release");
            location.push("Endless Sky.app");
            if !location.exists() {
                bail!("Binary is missing!");
            }
            Ok(Self { path: location })
        }

        pub fn command(&self) -> Command {
            Command::new("open").arg(&self.path)
        }

        pub fn file(&self) -> &Path {
            &self.path
        }
    }
}

use os::Binary;

pub static INSTANCE_DIR: Lazy<PathBuf> = Lazy::new(|| {
    let appdirs = directories::ProjectDirs::from("", "", "esmanager").unwrap();
    appdirs.data_dir().to_owned()
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub name: String,
    data: PathBuf,
    version: RepoVersion,
    binary: Binary,
}

impl Instance {
    pub  fn create(name: String, version: RepoVersion) -> Result<Self> {
        let location = INSTANCE_DIR.join(&name);
        let es_location = location.join("endless-sky");
        let repo = if !es_location.exists() {
            Repository::clone(&version.repo, &es_location)?
        } else {
            Repository::open(&es_location)?
        };
        version.checkout(&repo)?;
        let binary = Binary::compile(es_location)?;
        let mut this = Self {
            data: location.join("data"),
            version,
            name,
            binary,
        };
        std::fs::write(
            location.join("instance.json"),
            serde_json::to_string_pretty(&this)?,
        )?;
        this.ensure()?;
        Ok(this)
    }

    pub  fn open(location: PathBuf) -> Result<Self> {
        let text = std::fs::read_to_string(location.join("instance.json"))?;
        let mut this: Instance = serde_json::from_str(&text)?;
        this.ensure()?;
        Ok(this)
    }

    pub  fn ensure(&mut self) -> Result<()> {
        let saves = self.data.join("saves");
        if !saves.exists() {
            std::fs::create_dir_all(saves)?;
        }
        if !self.binary.file().exists() {
            self.binary = Binary::compile(
                self.data
                    .parent()
                    .ok_or_else(|| anyhow!("Data has no parent!"))?
                    .join("endless-sky"),
            )?;
        }
        Ok(())
    }

    pub  fn launch(&self) -> Result<i32> {
        self.binary
            .command()
            .arg("-c")
            .arg(&self.data)
            .spawn()?
            .wait()?
            .code()
            .ok_or_else(|| anyhow!("No exit code!"))
    }
}
