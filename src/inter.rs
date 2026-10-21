use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct License {
    pub key: String,
    pub name: String,
    #[serde(rename = "spdx_id")]
    pub spdx_id: String,
    pub url: String,
    #[serde(rename = "node_id")]
    pub node_id: String,
    #[serde(rename = "html_url")]
    pub html_url: String,
    pub description: String,
    pub implementation: String,
    pub permissions: Vec<String>,
    pub conditions: Vec<String>,
    pub limitations: Vec<String>,
    pub body: String,
    pub featured: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Licenses {
    pub key: String,
    pub name: String,
    #[serde(rename = "spdx_id")]
    pub spdx_id: String,
    pub url: String,
    #[serde(rename = "node_id")]
    pub node_id: String,
}
