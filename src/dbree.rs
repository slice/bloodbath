use anyhow::{anyhow, Result};
use http::Uri;
use isahc::{prelude::*, HttpClient};
use scraper::{ElementRef, Html, Selector};

#[derive(Clone, Debug)]
pub struct DbreeSearchResult {
    pub size: String,
    pub file: DbreeFile,
}

#[derive(Clone, Debug)]
pub struct DbreeFile {
    pub id: String,
    pub name: String,
}

pub struct Dbree {
    pub base_uri: Uri,
    pub client: HttpClient,
}

#[derive(Clone, Debug)]
pub struct DbreeSearch<'a> {
    pub query: &'a str,
    pub offset: u32,
}

impl Dbree {
    pub fn new(base_uri: Uri) -> Result<Self> {
        let client = HttpClient::builder()
            .timeout(std::time::Duration::from_secs(30))
            .redirect_policy(isahc::config::RedirectPolicy::Limit(5))
            .cookies()
            .build()?;

        Ok(Dbree { base_uri, client })
    }

    pub fn search(&self, search: &DbreeSearch) -> Result<Vec<DbreeSearchResult>> {
        let mut base_parts = self.base_uri.clone().into_parts();

        let path_and_query = format!("/s/{}?page={}", search.query, search.offset);
        base_parts.path_and_query = Some(path_and_query.try_into()?);
        let uri = Uri::from_parts(base_parts)?;

        let mut response = self.client.get(uri)?;

        lazy_static::lazy_static! {
            static ref RESULT_SELECTOR: Selector = Selector::parse("ul.list-group li.list-group-item").unwrap();
            static ref RESULT_BADGE_SELECTOR: Selector = Selector::parse("span.badge").unwrap();
            static ref RESULT_A_SELECTOR: Selector = Selector::parse("a").unwrap();
        }
        let document = Html::parse_document(&response.text()?);

        let transform_result = |element_ref: ElementRef| -> Result<DbreeSearchResult> {
            let badge = element_ref
                .select(&RESULT_BADGE_SELECTOR)
                .next()
                .ok_or_else(|| anyhow!("no badge found"))?;
            let badge_text = badge
                .text()
                .next()
                .ok_or_else(|| anyhow!("badge has no text"))?;

            let a = element_ref
                .select(&RESULT_A_SELECTOR)
                .next()
                .ok_or_else(|| anyhow!("search item has no <a>"))?;
            let filename = a
                .text()
                .next()
                .ok_or_else(|| anyhow!("search item's <a> has no text"))?;
            let href = a
                .value()
                .attr("href")
                .ok_or_else(|| anyhow!("search item's <a> has no href"))?;
            let id = &href[3..];

            Ok(DbreeSearchResult {
                size: String::from(badge_text),
                file: DbreeFile {
                    id: String::from(id),
                    name: String::from(filename),
                },
            })
        };

        document
            .select(&RESULT_SELECTOR)
            .into_iter()
            .map(transform_result)
            .collect::<Result<_>>()
    }
}
