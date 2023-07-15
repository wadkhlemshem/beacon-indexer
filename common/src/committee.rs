use serde::{Deserialize, Serialize};

use crate::util::{deserialize_num, deserialize_vec_num};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Committee {
    #[serde(deserialize_with = "deserialize_num")]
    pub index: u8,
    #[serde(deserialize_with = "deserialize_num")]
    pub slot: u64,
    #[serde(deserialize_with = "deserialize_vec_num")]
    pub validators: Vec<u64>,
}
