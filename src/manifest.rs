use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

use serde::de::{self, Deserialize};

use crate::utils;

pub type ModDependencies = BTreeMap<String, ModDependency>;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ModioManifest {
    pub game: Game,
    pub mods: Option<ModDependencies>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum Identifier {
    Id(u32),
    NameId(String),
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Game {
    pub id: Identifier,
    pub with_dependencies: Option<bool>,
}

// {{{ impl Deserialize for Identifier
impl<'de> Deserialize<'de> for Identifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Identifier;

            fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt.write_str("a string or an integer")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Identifier::NameId(s.to_string()))
            }

            fn visit_i64<E>(self, u: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Identifier::Id(u as u32))
            }

            fn visit_u64<E>(self, u: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Identifier::Id(u as u32))
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}
// }}}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum ModDependency {
    Simple(Identifier),
    Detailed(DetailedModDependency),
}

impl ModDependency {
    pub fn id(&self) -> &Identifier {
        match *self {
            ModDependency::Simple(ref id) => id,
            ModDependency::Detailed(ref mod_) => &mod_.id,
        }
    }

    pub fn file(&self) -> Option<u32> {
        match *self {
            ModDependency::Simple(_) => None,
            ModDependency::Detailed(ref m) => m.file,
        }
    }

    pub fn version(&self) -> Option<&String> {
        match *self {
            ModDependency::Simple(_) => None,
            ModDependency::Detailed(ref mod_) => mod_.version.as_ref(),
        }
    }
}

// {{{ impl Deserialize for ModDependency
impl<'de> Deserialize<'de> for ModDependency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = ModDependency;

            fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt.write_str("a string or an integer")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ModDependency::Simple(Identifier::NameId(s.to_string())))
            }

            fn visit_i64<E>(self, u: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ModDependency::Simple(Identifier::Id(u as u32)))
            }

            fn visit_u64<E>(self, u: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ModDependency::Simple(Identifier::Id(u as u32)))
            }

            fn visit_map<V>(self, map: V) -> Result<Self::Value, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mvd = de::value::MapAccessDeserializer::new(map);
                DetailedModDependency::deserialize(mvd).map(ModDependency::Detailed)
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}
// }}}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DetailedModDependency {
    pub id: Identifier,
    pub with_dependencies: Option<bool>,
    pub file: Option<u32>,
    pub version: Option<String>,
}

pub fn read(path: &Path) -> Result<ModioManifest, Box<dyn std::error::Error>> {
    let content = utils::read(&path)?;
    parse(&content, &path)
}

pub fn parse(content: &str, path: &Path) -> Result<ModioManifest, Box<dyn std::error::Error>> {
    let manifest = toml::from_str(&content)
        .map_err(|_| format!("failed to parse manifest at `{}`", path.display()))?;
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifier() {
        #[derive(Debug, Eq, PartialEq, Deserialize)]
        struct Test {
            id: Identifier,
        }
        let actual = toml::from_str("id=1");
        let expected = Test {
            id: Identifier::Id(1),
        };
        assert!(actual.is_ok());
        assert_eq!(expected, actual.unwrap());

        let actual = toml::from_str(r#"id="game1""#);
        let expected = Test {
            id: Identifier::NameId("game1".to_string()),
        };
        assert!(actual.is_ok());
        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn mod_dependencies() {
        let mod1 = ModDependency::Simple(Identifier::Id(1));
        let mod2 = ModDependency::Simple(Identifier::NameId("mod2".to_string()));
        let mod3 = ModDependency::Detailed(DetailedModDependency {
            id: Identifier::Id(3),
            with_dependencies: Some(true),
            file: None,
            version: None,
        });
        let mut expected = ModDependencies::new();
        expected.insert("mod1".to_string(), mod1);
        expected.insert("mod2".to_string(), mod2);
        expected.insert("mod3".to_string(), mod3);

        let raw = r#"
        mod1 = 1
        mod2 = "mod2"
        mod3 = { id = 3, with-dependencies = true }
        "#;

        let actual = toml::from_str(raw);
        assert!(actual.is_ok());
        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn manifest() {
        let raw = r#"
        [game]
        id = "gameone"

        [mods]
        mod1 = 1
        mod2 = "mod2"
        mod3 = { id = 3, with-dependencies = true }

        [mods.mod4]
        id = "mod4"

        [mods.mod5]
        id = "mod5"
        version = "1.2"
        "#;

        let mod1 = ModDependency::Simple(Identifier::Id(1));
        let mod2 = ModDependency::Simple(Identifier::NameId("mod2".to_string()));
        let mod3 = ModDependency::Detailed(DetailedModDependency {
            id: Identifier::Id(3),
            with_dependencies: Some(true),
            file: None,
            version: None,
        });
        let mod4 = ModDependency::Detailed(DetailedModDependency {
            id: Identifier::NameId("mod4".to_string()),
            with_dependencies: None,
            file: None,
            version: None,
        });
        let mod5 = ModDependency::Detailed(DetailedModDependency {
            id: Identifier::NameId("mod5".to_string()),
            with_dependencies: None,
            file: None,
            version: Some("1.2".to_string()),
        });
        let mut mods = ModDependencies::new();
        mods.insert("mod1".to_string(), mod1);
        mods.insert("mod2".to_string(), mod2);
        mods.insert("mod3".to_string(), mod3);
        mods.insert("mod4".to_string(), mod4);
        mods.insert("mod5".to_string(), mod5);
        let expected = ModioManifest {
            game: Game {
                id: Identifier::NameId("gameone".to_string()),
                with_dependencies: None,
            },
            mods: Some(mods),
        };

        let actual = toml::from_str(raw);
        assert!(actual.is_ok());
        assert_eq!(expected, actual.unwrap());
    }
}

// vim: fdm=marker
