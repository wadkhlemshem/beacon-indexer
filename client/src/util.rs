use std::{fmt::Display, str::FromStr};

use serde::Deserialize;

pub fn deserialize_num<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: std::str::FromStr,
    <T as FromStr>::Err: Display,
{
    let s = String::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}

pub fn deserialize_vec_num<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: std::str::FromStr,
    <T as FromStr>::Err: Display,
{
    let s = <Vec<String>>::deserialize(deserializer)?;
    s.into_iter()
        .map(|s| s.parse().map_err(serde::de::Error::custom))
        .collect()
}
