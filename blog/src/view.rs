use crate::{
    bindings::{
        exports::nur::cms::http_handler::{PluginError, Response},
        nur::cms::types::Header,
    },
    config::BlogConfig,
    content::{
        SearchResult, article_list, author_name, entry_html, index_page, media_url, summary_html,
        title,
    },
};
use maud::{DOCTYPE, Markup, PreEscaped, html};
use serde_json::Value;

const ASSET_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn home_page(config: &BlogConfig, offset: usize) -> Result<Markup, PluginError> {
    let articles = article_list(&config.article_type, config.posts_per_page, offset)?;
    let (articles, next_exists) = article_page_items(articles, config.posts_per_page);
    let hero = config
        .index_page_slug
        .as_deref()
        .map(|slug| index_hero(config, slug))
        .transpose()?
        .unwrap_or_else(|| default_hero(config));
    let page = offset / config.posts_per_page + 1;

    Ok(html! {
        (hero)
        section class="blog-list" aria-label="Latest articles" {
            @if articles.is_empty() {
                p class="empty-state" { "No articles have been published yet." }
            } @else {
                @for article in &articles { (article_card(config, article)) }
            }
        }
        (pagination(page, next_exists))
    })
}

fn article_page_items(mut articles: Vec<Value>, page_size: usize) -> (Vec<Value>, bool) {
    let next_exists = articles.len() > page_size;
    articles.truncate(page_size);
    (articles, next_exists)
}

pub fn article_page(article: &Value) -> Markup {
    html! {
        article class="article prose" {
            (metadata(article))
            h1 { (title(article, "Untitled article")) }
            @if let Some(url) = media_url(article) {
                img class="article-cover" src=(url) alt="";
            }
            div class="article-content" { (PreEscaped(entry_html(article))) }
            (tags(article))
        }
    }
}

pub fn content_page(page: &Value) -> Markup {
    html! {
        article class="content-page prose" {
            h1 { (title(page, "Page")) }
            (PreEscaped(entry_html(page)))
        }
    }
}

pub fn search_page(query: &str, results: &[SearchResult]) -> Markup {
    html! {
        section class="search-page" {
            p class="eyebrow" { "Search" }
            h1 { "Search the blog" }
            form class="search-page-form" action="/search" method="get" {
                label class="visually-hidden" for="search-page-input" { "Search articles and pages" }
                input id="search-page-input" type="search" name="q" value=(query)
                    placeholder="Search articles and pages" minlength="2" maxlength="120" required;
                button type="submit" { "Search" }
            }
            (search_results(query, results))
        }
    }
}

pub fn search_results(query: &str, results: &[SearchResult]) -> Markup {
    html! {
        div class="search-results" {
            @if query.chars().count() < 2 {
                p class="search-state" { "Enter at least two characters." }
            } @else if results.is_empty() {
                p class="search-state" { "No results for “" (query) "”." }
            } @else {
                p class="search-count" { (results.len()) " results" }
                ul {
                    @for result in results {
                        li {
                            a href=(&result.href) {
                                span { (&result.title) }
                                small { (result.kind) }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn response(
    config: &BlogConfig,
    title: String,
    content: Markup,
) -> Result<Response, PluginError> {
    Ok(Response {
        status: 200,
        headers: vec![Header {
            name: "content-type".into(),
            value: "text/html; charset=utf-8".into(),
        }],
        body: document(config, &title, content).into_bytes(),
    })
}

fn index_hero(config: &BlogConfig, slug: &str) -> Result<Markup, PluginError> {
    let entry = index_page(&config.page_type, slug)?;
    Ok(entry.map_or_else(
        || default_hero(config),
        |entry| {
            html! {
                section class="hero" {
                    p class="eyebrow" { (&config.site_name) }
                    h1 { (title(&entry, &config.site_name)) }
                    div class="hero-copy" { (PreEscaped(entry_html(&entry))) }
                }
            }
        },
    ))
}

fn default_hero(config: &BlogConfig) -> Markup {
    html! {
        section class="hero" {
            p class="eyebrow" { (&config.site_name) }
            h1 { (&config.site_description) }
        }
    }
}

fn article_card(config: &BlogConfig, article: &Value) -> Markup {
    let title = title(article, "Untitled article");
    let slug = article
        .get("slug")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let href = format!("/{}/{slug}", config.article_type);
    html! {
        article class="article-card" {
            @if let Some(url) = media_url(article) {
                a class="article-card-image" href=(&href) aria-label=(format!("Read {title}")) {
                    img src=(url) alt="" loading="lazy";
                }
            }
            div class="article-card-body" {
                (metadata(article))
                h2 { a href=(&href) { (title) } }
                @if let Some(summary) = summary_html(article) {
                    div class="article-summary" { (PreEscaped(summary)) }
                }
                a class="read-more" href=(&href) { "Read article" span aria-hidden="true" { " →" } }
            }
        }
    }
}

fn metadata(entry: &Value) -> Markup {
    let date = entry.get("created_at").and_then(Value::as_str);
    let category = entry
        .get("category")
        .and_then(|value| value.get("name"))
        .and_then(Value::as_str);
    let authors = entry
        .get("authors")
        .and_then(Value::as_array)
        .map(|authors| {
            authors
                .iter()
                .filter_map(author_name)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .filter(|authors| !authors.is_empty());
    html! {
        @if date.is_some() || category.is_some() || authors.is_some() {
            p class="metadata" {
                @if let Some(category) = category { span class="category" { (category) } }
                @if let Some(date) = date { time datetime=(date) { (date) } }
                @if let Some(authors) = authors { span { (authors) } }
            }
        }
    }
}

fn tags(entry: &Value) -> Markup {
    let tags = entry
        .get("tags")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|tag| tag.get("name").and_then(Value::as_str))
        .collect::<Vec<_>>();
    html! {
        @if !tags.is_empty() {
            ul class="tag-list" aria-label="Tags" { @for tag in tags { li { (tag) } } }
        }
    }
}

fn pagination(page: usize, next_exists: bool) -> Markup {
    html! {
        @if page > 1 || next_exists {
            nav class="pagination" aria-label="Article pages" {
                @if page > 1 { a href=(page_url(page - 1)) { "← Newer" } }
                @if next_exists { a href=(page_url(page + 1)) { "Older →" } }
            }
        }
    }
}

fn page_url(page: usize) -> String {
    if page == 1 {
        "/".into()
    } else {
        format!("/page/{page}")
    }
}

fn document(config: &BlogConfig, title: &str, content: Markup) -> String {
    let favicon_url = config
        .favicon_url
        .clone()
        .unwrap_or_else(|| asset_url("favicon.svg"));
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                meta name="description" content=(&config.site_description);
                title { (title) " · " (&config.site_name) }
                link rel="icon" href=(favicon_url);
                link rel="stylesheet" href=(asset_url("blog.min.css"));
                link rel="stylesheet" href=(asset_url("theme-overrides.css"));
            }
            body {
                div class="site-shell" {
                    aside class="site-sidebar" {
                        a class="site-brand" href="/" { (&config.site_name) }
                        nav class="site-navigation" aria-label="Main navigation" {
                            @for item in &config.navigation { a href=(&item.href) { (&item.label) } }
                        }
                        (search_trigger())
                    }
                    div class="site-main" {
                        header class="site-header" {
                            a class="site-header-brand" href="/" { (&config.site_name) }
                            p { (&config.site_description) }
                            (search_trigger())
                            nav class="site-header-navigation" aria-label="Main navigation" {
                                @for item in &config.navigation { a href=(&item.href) { (&item.label) } }
                            }
                        }
                        main { (content) }
                        footer class="site-footer" { p { "© " (&config.site_name) } }
                    }
                }
                (search_dialog())
                script src=(asset_url("blog.min.js")) defer {}
            }
        }
    }
    .into_string()
}

fn search_trigger() -> Markup {
    html! {
        a class="search-trigger" href="/search" data-search-open aria-haspopup="dialog" {
            (search_icon())
            span { "Search" }
            kbd { "⌘K" }
        }
    }
}

fn search_dialog() -> Markup {
    html! {
        dialog class="search-dialog" id="search-dialog" aria-label="Search" {
            div class="search-panel" {
                form class="search-modal-form" action="/search" method="get" {
                    (search_icon())
                    label class="visually-hidden" for="search-modal-input" { "Search articles and pages" }
                    input id="search-modal-input" type="search" name="q"
                        placeholder="Search articles and pages" autocomplete="off"
                        minlength="2" maxlength="120" required;
                    button class="search-close" type="button" data-search-close aria-label="Close search" { "×" }
                }
                div class="search-modal-results" aria-live="polite" {
                    p class="search-state" { "Enter at least two characters." }
                }
                footer { span { "Search articles and pages" } kbd { "esc" } }
            }
        }
    }
}

fn search_icon() -> Markup {
    html! {
        svg aria-hidden="true" viewBox="0 0 24 24" {
            circle cx="11" cy="11" r="7" {}
            path d="m16 16 5 5" {}
        }
    }
}

fn asset_url(filename: &str) -> String {
    format!("/p/blog/assets/{filename}?v={ASSET_VERSION}")
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::article_page_items;

    #[test]
    fn extra_article_controls_next_page_without_being_rendered() {
        let articles = (0..7).map(|id| json!({ "id": id })).collect();
        let (articles, next_exists) = article_page_items(articles, 6);

        assert_eq!(articles.len(), 6);
        assert!(next_exists);

        let (articles, next_exists) = article_page_items(articles, 6);
        assert_eq!(articles.len(), 6);
        assert!(!next_exists);
    }
}
