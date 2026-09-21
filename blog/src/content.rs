use crate::bindings::{exports::nur::cms::http_handler::PluginError, nur::cms::content};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use serde_json::Value;

const ARTICLE_FIELDS: &str = "title,slug,created_at,media,author.first_name,author.last_name,category.name,category.slug,tags,node.html";
const SEARCH_LIMIT_PER_TYPE: usize = 6;

#[derive(Clone)]
pub struct SearchResult {
    pub title: String,
    pub href: String,
    pub kind: &'static str,
}

pub fn article_list(
    article_type: &str,
    page_size: usize,
    offset: usize,
) -> Result<Vec<Value>, PluginError> {
    let limit = page_size.saturating_add(1);
    entries(&format!(
        "type={article_type}&fields={ARTICLE_FIELDS}&node_limit=1&character_limit=360&ordering=created_at+DESC&limit={limit}&offset={offset}"
    ))
}

pub fn entry(content_type: &str, slug: &str) -> Result<Option<Value>, PluginError> {
    entries(&format!(
        "type={content_type}&slug={slug}&fields={ARTICLE_FIELDS}&ordering=created_at+DESC&limit=1"
    ))
    .map(|entries| entries.into_iter().next())
}

pub fn index_page(content_type: &str, slug: &str) -> Result<Option<Value>, PluginError> {
    entries(&format!(
        "type={content_type}&slug={slug}&fields=title,node.html&limit=1"
    ))
    .map(|entries| entries.into_iter().next())
}

pub fn search(
    article_type: &str,
    page_type: &str,
    term: &str,
) -> Result<Vec<SearchResult>, PluginError> {
    let articles = search_type(article_type, term, true)?;
    let pages = if page_type == article_type {
        Vec::new()
    } else {
        search_type(page_type, term, false)?
    };
    let mut results = Vec::with_capacity(articles.len() + pages.len());
    let count = articles.len().max(pages.len());
    for index in 0..count {
        if let Some(article) = articles.get(index) {
            results.push(article.clone());
        }
        if let Some(page) = pages.get(index) {
            results.push(page.clone());
        }
    }
    results.truncate(10);
    Ok(results)
}

fn search_type(
    content_type: &str,
    term: &str,
    article: bool,
) -> Result<Vec<SearchResult>, PluginError> {
    let term = utf8_percent_encode(term, NON_ALPHANUMERIC);
    entries(&format!(
        "type={content_type}&fields=title,slug&search={term}&limit={SEARCH_LIMIT_PER_TYPE}"
    ))
    .map(|entries| {
        entries
            .into_iter()
            .filter_map(|entry| {
                let slug = entry.get("slug")?.as_str()?;
                let title = title(&entry, "Untitled").to_owned();
                Some(SearchResult {
                    title,
                    href: if article {
                        format!("/{content_type}/{slug}")
                    } else {
                        format!("/{slug}")
                    },
                    kind: if article { "Article" } else { "Page" },
                })
            })
            .collect()
    })
}

pub fn entries(query: &str) -> Result<Vec<Value>, PluginError> {
    let bytes = content::published_entries(query, content::OutputType::Html)?;
    let response: Value = serde_json::from_slice(&bytes)
        .map_err(|_| PluginError::Failed("CMS returned invalid content data".into()))?;
    Ok(response
        .get("results")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default())
}

pub fn entry_html(entry: &Value) -> String {
    entry
        .get("nodes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .flat_map(node_html)
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn summary_html(entry: &Value) -> Option<String> {
    let content = entry_html(entry);
    (!content.trim().is_empty()).then_some(content)
}

pub fn media_url(entry: &Value) -> Option<String> {
    let media = entry.get("media")?;
    let path = media.get("path")?.as_str()?.trim_end_matches('/');
    let filename = media.get("filename")?.as_str()?;
    if path.is_empty() || filename.is_empty() {
        return None;
    }

    Some(if path == "/" {
        format!("/{filename}")
    } else {
        format!("{path}/{filename}")
    })
}

pub fn title<'a>(entry: &'a Value, fallback: &'a str) -> &'a str {
    entry
        .get("title")
        .and_then(Value::as_str)
        .filter(|title| !title.is_empty())
        .unwrap_or(fallback)
}

pub fn author_name(author: &Value) -> Option<String> {
    let first = author
        .get("first_name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let last = author
        .get("last_name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let name = format!("{first} {last}").trim().to_owned();
    (!name.is_empty()).then_some(name)
}

fn node_html(node: &Value) -> Vec<String> {
    if let Some(blocks) = node.get("blocks").and_then(Value::as_array) {
        return blocks.iter().flat_map(node_html).collect();
    }

    node.get("html")
        .and_then(Value::as_str)
        .filter(|html| !html.trim().is_empty())
        .map(|html| vec![html.to_owned()])
        .unwrap_or_default()
}
