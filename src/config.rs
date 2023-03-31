use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub database_path: String,

    #[serde(default = "default_dbree_base_uri")]
    pub dbree_base_uri: String,

    #[serde(default)]
    pub ignored_keywords: Vec<String>,

    pub discord: DiscordConfig,

    pub cookies: HashMap<String, String>,

    pub queries: Vec<String>,
}

fn default_dbree_base_uri() -> String {
    String::from("https://dbree.org")
}

#[derive(Deserialize)]
pub struct DiscordConfig {
    pub webhook_uri: String,
}
