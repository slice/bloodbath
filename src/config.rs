use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub database_path: String,

    #[serde(default = "default_dbree_base_uri")]
    pub dbree_base_uri: String,

    pub discord: DiscordConfig,

    pub ddos_guard: DdosGuardConfig,

    pub queries: Vec<String>,
}

fn default_dbree_base_uri() -> String {
    String::from("https://dbree.org")
}

#[derive(Deserialize)]
pub struct DiscordConfig {
    pub webhook_uri: String,
}

#[derive(Deserialize)]
pub struct DdosGuardConfig {
    pub ddg1: String,
    pub ddg2: String,
    pub ddgid: String,
}
