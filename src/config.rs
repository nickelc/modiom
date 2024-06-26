use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use home::home_dir;
use serde::{Deserialize, Serialize};

use modio::auth::Credentials;

use crate::Result;

#[derive(Debug)]
pub struct Config {
    cwd: PathBuf,
    home_dir: PathBuf,
    test_env: bool,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct TomlConfig {
    #[serde(rename = "host")]
    #[serde(default)]
    hosts: BTreeMap<String, TomlCredentials>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TomlCredentials {
    api_key: String,
    token: String,
}

impl Config {
    pub fn new(cwd: PathBuf, home_dir: PathBuf) -> Self {
        Self {
            cwd,
            home_dir,
            test_env: false,
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Result<Config> {
        let cwd = env::current_dir()?;
        let homedir = home_dir().ok_or("Couldn't find your home directory.")?;
        Ok(Config::new(cwd, homedir.join(".modio")))
    }

    pub fn configure(&mut self, test_env: bool) -> Result<()> {
        self.test_env = test_env;
        Ok(())
    }

    pub fn home(&self) -> &Path {
        &self.home_dir
    }

    pub fn cwd(&self) -> &Path {
        &self.cwd
    }

    pub fn host(&self) -> &str {
        if self.test_env {
            "https://api.test.mod.io/v1"
        } else {
            "https://api.mod.io/v1"
        }
    }

    pub fn auth_token(&self) -> Result<Option<Credentials>> {
        let config = self.load_config()?;
        if let Some(creds) = config.hosts.get(self.host()) {
            Ok(Some(Credentials {
                api_key: creds.api_key.clone(),
                token: Some(modio::auth::Token {
                    value: creds.token.clone(),
                    expired_at: None,
                }),
            }))
        } else {
            Ok(None)
        }
    }

    fn load_config(&self) -> Result<TomlConfig> {
        fs::create_dir_all(&self.home_dir)?;

        let path = self.home_dir.join("credentials");
        let file = fs::File::open(path).and_then(|file| file.metadata().map(|md| (file, md)));

        match file {
            Ok((mut file, md)) if md.is_file() => {
                use std::io::Read;

                let mut content = String::new();
                file.read_to_string(&mut content)?;

                Ok(toml::from_str(&content)?)
            }
            Ok(_) | Err(_) => Ok(TomlConfig::default()),
        }
    }

    pub fn save_credentials(&self, api_key: String, token: String) -> Result<()> {
        let mut config = self.load_config()?;

        let creds = TomlCredentials { api_key, token };
        config.hosts.insert(self.host().to_owned(), creds);

        let content = toml::to_string(&config)?;
        let path = self.home_dir.join("credentials");
        match fs::write(&path, content) {
            Ok(()) => Ok(()),
            Err(e) => Err(format!("Failed to write {}: {}", path.display(), e).into()),
        }
    }
}
