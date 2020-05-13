use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use dirs::home_dir;
use serde::{Deserialize, Serialize};

use modio::auth::Credentials;

use crate::Result;

#[derive(Debug)]
pub struct Config {
    cwd: PathBuf,
    home_dir: PathBuf,
    test_env: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct TomlConfig {
    #[serde(rename = "host")]
    hosts: Option<BTreeMap<String, TomlCredentials>>,
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

    pub fn default() -> Result<Config> {
        let cwd = env::current_dir()?;
        let homedir = home_dir().ok_or_else(|| "Couldn't find your home directory.")?;
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
        let mut config = self.load_config()?;
        let hosts = config.hosts.get_or_insert_with(Default::default);
        if let Some(creds) = hosts.get(self.host()) {
            Ok(Some(Credentials {
                api_key: creds.api_key.to_owned(),
                token: Some(modio::auth::Token {
                    value: creds.token.to_owned(),
                    expired_at: None,
                }),
            }))
        } else {
            Ok(None)
        }
    }

    fn load_config(&self) -> Result<TomlConfig> {
        fs::create_dir_all(&self.home_dir)?;
        let content = fs::read_to_string(self.home_dir.join("credentials"))?;
        Ok(toml::from_str(&content)?)
    }

    pub fn save_credentials(&self, api_key: String, token: String) -> Result<()> {
        let mut config = self.load_config()?;

        let hosts = config.hosts.get_or_insert_with(Default::default);
        hosts.insert(self.host().to_owned(), TomlCredentials { api_key, token });

        let content = toml::to_string(&config)?;
        fs::write(self.home_dir.join("credentials"), content)?;
        Ok(())
    }
}
