use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::{Path, PathBuf};

use dirs::home_dir;
use lazycell::LazyCell;

use cfg::{Config as Cfg, ConfigError};
use cfg::{Environment, File, FileFormat};

use errors::ModiomResult;

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
        let homedir = home_dir().ok_or(format_err!("Couldn't find your home directory."))?;
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

    pub fn auth_token(&self) -> ModiomResult<Option<String>> {
        if self.test_env {
            Ok(self.get_string("auth.test.token")?)
        } else {
            Ok(self.get_string("auth.token")?)
        }
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
        if let Ok(Some(old_token)) = self.auth_token() {
            if token == old_token {
                return Ok(());
            }
        }

        let mut table: ::toml::value::Table = BTreeMap::new();
        table.insert("token".into(), token.into());

        let table = if self.test_env {
            let mut env_table: ::toml::value::Table = BTreeMap::new();
            env_table.insert("test".into(), table.into());
            env_table
        } else {
            table
        };

        let mut toml: ::toml::value::Table = BTreeMap::new();
        toml.insert("auth".into(), table.into());
        let content = ::toml::Value::Table(toml).to_string();

        fs::create_dir_all(&self.home_dir)?;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(self.home_dir.join("credentials"))?;
        file.seek(SeekFrom::Start(0))?;
        file.write_all(content.as_bytes())?;
        file.set_len(content.len() as u64)?;
        Ok(())
    }
}
