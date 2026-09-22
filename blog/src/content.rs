use crate::bindings::{exports::nur::cms::http_handler::PluginError, nur::cms::content};
use maud::Render;
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use rust_i18n::t;
use serde::Deserialize;

const ARTICLE_FIELDS: &str =
    "title,slug,created_at,media,author.first_name,author.last_name,category.name,tags,node.html";
const SEARCH_LIMIT_PER_TYPE: usize = 6;

pub struct SearchResult {
    pub title: String,
    pub href: String,
    pub article: bool,
}

pub struct ArticleList {
    pub entries: Vec<Entry>,
    pub total: usize,
}

#[derive(Deserialize)]
pub struct Entry {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub media: Option<Media>,
    #[serde(default)]
    pub category: Option<EntryCategory>,
    #[serde(default)]
    pub authors: Vec<Author>,
    #[serde(default)]
    pub tags: Vec<Tag>,
    #[serde(default)]
    nodes: Vec<ContentNode>,
}

#[derive(Deserialize)]
pub struct Media {
    path: String,
    filename: String,
}

#[derive(Deserialize)]
pub struct EntryCategory {
    pub name: String,
}

#[derive(Deserialize)]
pub struct Author {
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
}

#[derive(Deserialize)]
pub struct Tag {
    pub name: String,
}

#[derive(Deserialize)]
struct ContentNode {
    #[serde(default)]
    html: Option<String>,
    #[serde(default)]
    blocks: Vec<ContentNode>,
}

#[derive(Deserialize)]
struct EntryResponse {
    #[serde(default)]
    count: usize,
    #[serde(default)]
    results: Vec<Entry>,
}

#[derive(Deserialize)]
struct FacetResponse {
    #[serde(default)]
    categories: Vec<Category>,
}

#[derive(Deserialize)]
pub struct Category {
    pub name: String,
    pub slug: String,
    pub count: usize,
}

pub struct EntryHtml<'a>(&'a Entry);

impl Render for EntryHtml<'_> {
    fn render_to(&self, output: &mut String) {
        for node in &self.0.nodes {
            render_node_html(node, output);
        }
    }
}

impl Entry {
    pub fn title<'a>(&'a self, fallback: &'a str) -> &'a str {
        self.title
            .as_deref()
            .filter(|title| !title.is_empty())
            .unwrap_or(fallback)
    }

    pub fn html(&self) -> EntryHtml<'_> {
        EntryHtml(self)
    }

    pub fn has_html(&self) -> bool {
        self.nodes.iter().any(ContentNode::has_html)
    }

    pub fn media_url(&self) -> Option<String> {
        let media = self.media.as_ref()?;
        let path = media.path.trim_end_matches('/');
        if media.filename.is_empty() {
            return None;
        }
        Some(if path.is_empty() {
            format!("/{}", media.filename)
        } else {
            format!("{path}/{}", media.filename)
        })
    }
}

impl Author {
    pub fn is_empty(&self) -> bool {
        self.first_name.as_deref().unwrap_or_default().is_empty()
            && self.last_name.as_deref().unwrap_or_default().is_empty()
    }
}

impl ContentNode {
    fn has_html(&self) -> bool {
        self.html
            .as_deref()
            .is_some_and(|html| !html.trim().is_empty())
            || self.blocks.iter().any(Self::has_html)
    }
}

fn render_node_html(node: &ContentNode, output: &mut String) {
    if node.blocks.is_empty() {
        if let Some(html) = node.html.as_deref().filter(|html| !html.trim().is_empty()) {
            output.push_str(html);
        }
        return;
    }
    for block in &node.blocks {
        render_node_html(block, output);
    }
}

pub fn article_list(
    article_type: &str,
    locale: &str,
    category: Option<&str>,
    page_size: usize,
    offset: usize,
) -> Result<ArticleList, PluginError> {
    let category = category.map_or_else(String::new, |category| format!("&category={category}"));
    let response = entry_response(&format!(
        "type={article_type}&locale={locale}{category}&fields={ARTICLE_FIELDS}&node_limit=1&character_limit=360&ordering=created_at+DESC&limit={page_size}&offset={offset}"
    ))?;
    let total = response.count.max(response.results.len());
    Ok(ArticleList {
        entries: response.results,
        total,
    })
}

pub fn categories(article_type: &str, locale: &str) -> Result<Vec<Category>, PluginError> {
    let bytes = content::published_entry_facets(&format!("type={article_type}&locale={locale}"))?;
    serde_json::from_slice::<FacetResponse>(&bytes)
        .map(|response| response.categories)
        .map_err(|_| PluginError::Failed("CMS returned invalid content facets".into()))
}

pub fn entry(content_type: &str, locale: &str, slug: &str) -> Result<Option<Entry>, PluginError> {
    entries(&format!(
        "type={content_type}&locale={locale}&slug={slug}&fields={ARTICLE_FIELDS}&ordering=created_at+DESC&limit=1"
    ))
    .map(|entries| entries.into_iter().next())
}

pub fn index_page(
    content_type: &str,
    locale: &str,
    slug: &str,
) -> Result<Option<Entry>, PluginError> {
    entries(&format!(
        "type={content_type}&locale={locale}&slug={slug}&fields=media,node.html&limit=1"
    ))
    .map(|entries| entries.into_iter().next())
}

pub fn search(
    article_type: &str,
    page_type: &str,
    locale: &str,
    term: &str,
) -> Result<Vec<SearchResult>, PluginError> {
    let articles = search_type(article_type, locale, term, true)?;
    let pages = if page_type == article_type {
        Vec::new()
    } else {
        search_type(page_type, locale, term, false)?
    };
    let mut results = Vec::with_capacity((articles.len() + pages.len()).min(10));
    let mut articles = articles.into_iter();
    let mut pages = pages.into_iter();
    loop {
        let article = articles.next();
        let page = pages.next();
        if article.is_none() && page.is_none() {
            break;
        }
        results.extend(article);
        results.extend(page);
        if results.len() >= 10 {
            break;
        }
    }
    results.truncate(10);
    Ok(results)
}

fn search_type(
    content_type: &str,
    locale: &str,
    term: &str,
    article: bool,
) -> Result<Vec<SearchResult>, PluginError> {
    let term = utf8_percent_encode(term, NON_ALPHANUMERIC);
    entries(&format!(
        "type={content_type}&locale={locale}&fields=title,slug&search={term}&limit={SEARCH_LIMIT_PER_TYPE}"
    ))
    .map(|entries| {
        entries
            .into_iter()
            .filter_map(|entry| {
                let slug = entry.slug?;
                let title = entry
                    .title
                    .filter(|title| !title.is_empty())
                    .unwrap_or_else(|| t!("content.untitled", locale = locale).into_owned());
                Some(SearchResult {
                    title,
                    href: if article {
                        format!("/{content_type}/{slug}")
                    } else {
                        format!("/{slug}")
                    },
                    article,
                })
            })
            .collect()
    })
}

fn entries(query: &str) -> Result<Vec<Entry>, PluginError> {
    entry_response(query).map(|response| response.results)
}

fn entry_response(query: &str) -> Result<EntryResponse, PluginError> {
    let bytes = content::published_entries(query, content::OutputType::Html)?;
    serde_json::from_slice(&bytes)
        .map_err(|_| PluginError::Failed("CMS returned invalid content data".into()))
}

#[cfg(test)]
mod tests {
    use maud::{Markup, html};

    use super::{Entry, EntryResponse};

    #[test]
    fn ignores_unrequested_cms_fields_and_renders_nested_html_directly() {
        let response: EntryResponse = serde_json::from_str(
            r#"{"count":1,"results":[{"title":"Typed","slug":"typed","ignored":{"large":true},"nodes":[{"blocks":[{"html":"<p>One</p>"},{"html":"<p>Two</p>"}]}]}]}"#,
        )
        .expect("typed CMS response");
        let entry = &response.results[0];
        let rendered: Markup = html! { div { (entry.html()) } };

        assert_eq!(rendered.into_string(), "<div><p>One</p><p>Two</p></div>");
        assert!(entry.has_html());
    }

    #[test]
    fn missing_optional_fields_are_supported() {
        let entry: Entry = serde_json::from_str(r#"{"slug":"minimal"}"#).expect("minimal entry");

        assert_eq!(entry.title("Fallback"), "Fallback");
        assert!(!entry.has_html());
        assert!(entry.media_url().is_none());
    }
}
