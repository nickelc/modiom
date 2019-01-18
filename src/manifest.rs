use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

use serde::de::{self, Deserialize};

use crate::errors::ModiomResult;
use crate::utils;

pub type ModDependencies = BTreeMap<String, ModDependency>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ModioManifest {
    game: Identifier,
    with_dependencies: Option<bool>,
    mods: Option<ModDependencies>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Identifier {
    Id(u32),
    NameId(String),
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

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ModDependency {
    Simple(Identifier),
    Detailed(DetailedModDependency),
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DetailedModDependency {
    id: Identifier,
    with_dependencies: Option<bool>,
}

pub fn read(path: &Path) -> ModiomResult<ModioManifest> {
    utils::read(&path).and_then(|content| parse(&content, &path))
}

pub fn parse(content: &str, path: &Path) -> ModiomResult<ModioManifest> {
    toml::from_str(&content)
        .map_err(|_| format!("failed to parse manifest at `{}`", path.display()).into())
}

// vim: fdm=marker
