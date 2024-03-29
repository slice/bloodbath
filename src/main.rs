use anyhow::{Context, Result};
use bloodbath::config::Config;
use bloodbath::dbree::{Dbree, DbreeSearch, DbreeSearchResult};
use isahc::cookies::CookieBuilder;
use isahc::{prelude::*, Request};
use serde_json::Value;

struct App {
    config: Config,
    db: sled::Db,
    dbree: Dbree,
}

impl App {
    fn from_config(config: Config) -> Result<Self> {
        let db: sled::Db = sled::open(&config.database_path).unwrap();
        let dbree = Dbree::new(config.dbree_base_uri.parse()?)?;
        let jar = dbree.client.cookie_jar().unwrap();

        for (name, value) in config.cookies.iter() {
            jar.set(CookieBuilder::new(name, value).build()?, &dbree.base_uri)?;
        }

        Ok(App { config, db, dbree })
    }

    fn make_embed_for_search_result(&self, search_result: &DbreeSearchResult) -> Value {
        let file_url = format!(
            "{base_uri}/v/{id}",
            base_uri = self.config.dbree_base_uri,
            id = search_result.file.id
        );

        let timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

        serde_json::json!({
            "title": search_result.file.name,
            "url": file_url,
            "timestamp": timestamp,
            "footer": {
                "text": search_result.size
            }
        })
    }

    pub fn discover(&self, query: &DbreeSearch) -> Result<()> {
        eprintln!("*** discovering {:?}", query);
        let mut embeds: Vec<Value> = Vec::new();

        for result in self.dbree.search(query)? {
            let key = format!("seen:{}", result.file.id);

            if self.db.contains_key(&key)? {
                eprintln!("already seen {}, ignoring", result.file.id);
                continue;
            }

            println!("{}: {} ({})", result.file.id, result.file.name, result.size);

            if self
                .config
                .ignored_keywords
                .iter()
                .any(|ignored_keyword| result.file.name.contains(ignored_keyword))
            {
                eprintln!(
                    "ignoring {}, because it contains an ignored keyword",
                    result.file.id
                );
                continue;
            }

            embeds.push(self.make_embed_for_search_result(&result));

            self.db.insert(key, b"")?;
        }

        for chunk in embeds.chunks(10) {
            eprintln!("posting to webhook ({} in chunk)", chunk.len());

            let len = embeds.len();
            let message = format!(
                "Detected {len} new file{s} for query `{query}`.",
                len = len,
                s = if len == 1 { "" } else { "s" },
                query = query.query,
            );
            self.post_to_discord_webhook(serde_json::json!({
                "content": message,
                "embeds": Value::Array(chunk.to_vec()),
            }))?;
        }

        Ok(())
    }

    fn post_to_discord_webhook(&self, content: Value) -> Result<()> {
        let mut response = Request::post(&self.config.discord.webhook_uri)
            .header("User-Agent", "bloodbath/0.0 (+https://slice.zone)")
            .header("Content-Type", "application/json")
            .body(content.to_string())?
            .send()?;
        if response.status() != http::StatusCode::NO_CONTENT {
            eprintln!(
                "[ERROR] discord responded with {:?}: {}",
                response.status(),
                response.text()?
            );
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| String::from("./config.toml"));

    let config = std::fs::read_to_string(&config_path)
        .with_context(|| format!("failed to read config from {}", config_path))?;
    let config: Config = toml::from_str(&config).with_context(|| "failed to deserialize config")?;

    let app = App::from_config(config).with_context(|| "failed to initialize application")?;
    for query in &app.config.queries {
        app.discover(&DbreeSearch { query, offset: 0 })
            .with_context(|| format!("failed to query for \"{}\"", query))?;
    }

    Ok(())
}
