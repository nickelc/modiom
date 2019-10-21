use std::env;
use std::fs;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::{Path, PathBuf};

use cfg::{Config as Cfg, ConfigError};
use cfg::{Environment, File, FileFormat};
use dirs::home_dir;
use lazycell::LazyCell;
use toml::value::Table;
use toml::Value;

use modio::auth::Credentials;

use crate::errors::{Error, ModiomResult};

#[derive(Debug)]
pub struct Config {
    cwd: PathBuf,
    home_dir: PathBuf,
    inner: LazyCell<Cfg>,
    test_env: bool,
}

impl Config {
    pub fn new(cwd: PathBuf, home_dir: PathBuf) -> Self {
        Self {
            inner: LazyCell::new(),
            cwd,
            home_dir,
            test_env: false,
        }
    }

    pub fn default() -> ModiomResult<Config> {
        let cwd = env::current_dir()?;
        let homedir =
            home_dir().ok_or_else(|| format_err!("Couldn't find your home directory."))?;
        Ok(Config::new(cwd, homedir.join(".modio")))
    }

    pub fn configure(&mut self, test_env: bool) -> ModiomResult<()> {
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

    pub fn auth_token(&self) -> ModiomResult<Option<Credentials>> {
        (|| -> ModiomResult<_> {
            if self.test_env {
                Ok(self.get_string("auth.test.token")?)
            } else {
                Ok(self.get_string("auth.token")?)
            }
        })()
        .map(|t| t.map(|t| Credentials::Token(t, None)))
        .map_err(format_err!(map "failed to read authentication token"))
    }

    fn cfg(&self) -> ModiomResult<&Cfg> {
        self.inner.try_borrow_with(|| self.load_config())
    }

    fn load_config(&self) -> ModiomResult<Cfg> {
        let mut cfg = Cfg::new();
        let credentials: File<_> = self.home_dir.join("credentials").into();
        cfg.merge(credentials.format(FileFormat::Toml).required(true))?;
        cfg.merge(Environment::with_prefix("modio").separator("_"))?;
        Ok(cfg)
    }

    pub fn get_string(&self, key: &str) -> ModiomResult<Option<String>> {
        match self.cfg()?.get_str(key) {
            Ok(v) => Ok(Some(v)),
            Err(ConfigError::NotFound(_)) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn save_credentials(&self, token: String) -> ModiomResult<()> {
        fs::create_dir_all(&self.home_dir)?;
        let mut file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(self.home_dir.join("credentials"))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let mut toml: Value = contents
            .parse()
            .map_err(|e| Error::from(format!("Failed to load credentials: {}", e)))?;

        let (key, value) = if self.test_env {
            let mut table = Table::new();
            table.insert("token".into(), token.into());
            ("test".into(), table.into())
        } else {
            ("token".into(), token.into())
        };

        if let Some(table) = toml.as_table_mut() {
            let auth = table
                .entry("auth")
                .or_insert_with(|| Table::new().into());

            // Make sure an existing value is a table
            if !auth.is_table() {
                *auth = Table::new().into();
            }
            if let Some(table) = auth.as_table_mut() {
                table.insert(key, value);
            }
        }

        let contents = toml.to_string();
        file.seek(SeekFrom::Start(0))?;
        file.write_all(contents.as_bytes())?;
        file.set_len(contents.len() as u64)?;

        Ok(())
    }
}
