use crate::{
    bindings::{
        exports::nur::cms::http_handler::{PluginError, Response},
        nur::cms::types::Header,
    },
    config::BlogConfig,
    content::{Category, Entry, SearchResult, article_list, categories, index_page},
};
use maud::{DOCTYPE, Markup, html};
use rust_i18n::t;

const ASSET_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn home_page(config: &BlogConfig, offset: usize) -> Result<Markup, PluginError> {
    let article_list = article_list(
        &config.article_type,
        &config.default_locale,
        config.active_category.as_deref(),
        config.posts_per_page,
        offset,
    )?;
    let hero = config
        .index_page_slug
        .as_deref()
        .map(|slug| index_hero(config, slug))
        .transpose()?
        .unwrap_or_else(|| default_hero(config));
    let page = offset / config.posts_per_page + 1;
    let total_pages = article_list.total.div_ceil(config.posts_per_page);

    Ok(html! {
        (hero)
        section class="blog-list" aria-label=(t!("articles.latest", locale = &config.default_locale)) {
            @if article_list.entries.is_empty() {
                p class="empty-state" { (t!("articles.empty", locale = &config.default_locale)) }
            } @else {
                @for article in &article_list.entries { (article_card(config, article)) }
            }
        }
        (pagination(
            page,
            total_pages,
            config,
            config.active_category.as_deref(),
        ))
    })
}

pub fn article_page(article: &Entry, locale: &str) -> Markup {
    html! {
        article class="article prose" {
            (metadata(article))
            h1 { (article.title(&t!("article.untitled", locale = locale))) }
            @if let Some(url) = article.media_url() {
                img class="article-cover" src=(url) alt="";
            }
            div class="article-content" { (article.html()) }
            (tags(article, locale))
        }
    }
}

pub fn content_page(page: &Entry, locale: &str) -> Markup {
    html! {
        article class="content-page prose" {
            h1 { (page.title(&t!("content.page", locale = locale))) }
            (page.html())
        }
    }
}

pub fn search_page(query: &str, results: &[SearchResult], config: &BlogConfig) -> Markup {
    let locale = &config.default_locale;
    let action = localized_url("/search", locale, config);
    html! {
        section class="search-page" {
            p class="eyebrow" { (t!("search.label", locale = locale)) }
            h1 { (t!("search.title", locale = locale)) }
            form class="search-page-form" action=(action) method="get" {
                label class="visually-hidden" for="search-page-input" { (t!("search.description", locale = locale)) }
                input id="search-page-input" type="search" name="q" value=(query)
                    placeholder=(t!("search.description", locale = locale)) minlength="2" maxlength="120" required;
                button type="submit" { (t!("search.label", locale = locale)) }
            }
            (search_results(query, results, config))
        }
    }
}

pub fn search_results(query: &str, results: &[SearchResult], config: &BlogConfig) -> Markup {
    let locale = &config.default_locale;
    html! {
        div class="search-results" {
            @if query.chars().count() < 2 {
                p class="search-state" { (t!("search.minimum", locale = locale)) }
            } @else if results.is_empty() {
                p class="search-state" { (t!("search.none", locale = locale, query = query)) }
            } @else {
                p class="search-count" {
                    @if results.len() == 1 {
                        (t!("search.results.one", locale = locale))
                    } @else {
                        (t!("search.results.other", locale = locale, count = results.len()))
                    }
                }
                ul {
                    @for result in results {
                        li {
                            a href=(localized_url(&result.href, locale, config)) {
                                span { (&result.title) }
                                small {
                                    @if result.article {
                                        (t!("content.article", locale = locale))
                                    } @else {
                                        (t!("content.page", locale = locale))
                                    }
                                }
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
    let categories = categories(&config.article_type, &config.default_locale)?;
    Ok(Response {
        status: 200,
        headers: vec![
            Header {
                name: "content-type".into(),
                value: "text/html; charset=utf-8".into(),
            },
            Header {
                name: "content-language".into(),
                value: config.default_locale.clone(),
            },
        ],
        body: document(config, &title, content, &categories).into_bytes(),
    })
}

fn index_hero(config: &BlogConfig, slug: &str) -> Result<Markup, PluginError> {
    let entry = index_page(&config.page_type, &config.default_locale, slug)?;
    Ok(entry.map_or_else(
        || default_hero(config),
        |entry| {
            html! {
                section class="hero" {
                    @if let Some(url) = entry.media_url() {
                        img class="hero-image" src=(url) alt="";
                    }
                    div class="hero-body" {
                        p class="eyebrow" { (&config.site_name) }
                        h1 { (&config.site_description) }
                        div class="hero-copy" { (entry.html()) }
                    }
                }
            }
        },
    ))
}

fn default_hero(config: &BlogConfig) -> Markup {
    html! {
        section class="hero" {
            div class="hero-body" {
                p class="eyebrow" { (&config.site_name) }
                h1 { (&config.site_description) }
            }
        }
    }
}

fn article_card(config: &BlogConfig, article: &Entry) -> Markup {
    let untitled = t!("article.untitled", locale = &config.default_locale);
    let title = article.title(&untitled);
    let slug = article.slug.as_deref().unwrap_or_default();
    let href = localized_url(
        &format!("/{}/{slug}", config.article_type),
        &config.default_locale,
        config,
    );
    html! {
        article class="article-card" {
            @if let Some(url) = article.media_url() {
                a class="article-card-image" href=(&href)
                    aria-label=(t!("article.read_named", locale = &config.default_locale, title = title)) {
                    img src=(url) alt="" loading="lazy";
                }
            }
            div class="article-card-body" {
                (metadata(article))
                h2 { a href=(&href) { (title) } }
                @if article.has_html() {
                    div class="article-summary" { (article.html()) }
                }
                a class="read-more" href=(&href) {
                    (t!("article.read", locale = &config.default_locale))
                    span aria-hidden="true" { " →" }
                }
            }
        }
    }
}

fn metadata(entry: &Entry) -> Markup {
    let date = entry.created_at.as_deref();
    let category = entry
        .category
        .as_ref()
        .map(|category| category.name.as_str());
    let has_authors = entry.authors.iter().any(|author| !author.is_empty());
    html! {
        @if date.is_some() || category.is_some() || has_authors {
            p class="metadata" {
                @if let Some(category) = category { span class="category" { (category) } }
                @if let Some(date) = date {
                    time datetime=(date) data-local-datetime="" { (date) }
                }
                @if has_authors {
                    span {
                        @for (index, author) in entry.authors.iter().filter(|author| !author.is_empty()).enumerate() {
                            @if index > 0 { ", " }
                            @if let Some(first_name) = author.first_name.as_deref().filter(|name| !name.is_empty()) {
                                (first_name)
                                @if author.last_name.as_deref().is_some_and(|name| !name.is_empty()) { " " }
                            }
                            @if let Some(last_name) = author.last_name.as_deref().filter(|name| !name.is_empty()) {
                                (last_name)
                            }
                        }
                    }
                }
            }
        }
    }
}

fn tags(entry: &Entry, locale: &str) -> Markup {
    html! {
        @if !entry.tags.is_empty() {
            ul class="tag-list" aria-label=(t!("tags.label", locale = locale)) {
                @for tag in &entry.tags { li { (&tag.name) } }
            }
        }
    }
}

#[derive(Debug, PartialEq)]
enum PaginationItem {
    Page(usize),
    Ellipsis,
}

fn pagination(
    page: usize,
    total_pages: usize,
    config: &BlogConfig,
    category: Option<&str>,
) -> Markup {
    let locale = &config.default_locale;
    let items = pagination_items(page, total_pages);
    html! {
        @if total_pages > 1 {
            nav class="pagination" aria-label=(t!("pagination.label", locale = locale)) {
                @if page > 1 {
                    a class="pagination-newer" href=(page_url(page - 1, config, category)) {
                        (t!("pagination.newer", locale = locale))
                    }
                }
                ol class="pagination-pages" {
                    @for item in items {
                        @match item {
                            PaginationItem::Page(number) => {
                                li {
                                    @if number == page {
                                        span class="pagination-current" aria-current="page"
                                            aria-label=(t!("pagination.current", locale = locale, page = number)) {
                                            (number)
                                        }
                                    } @else {
                                        a href=(page_url(number, config, category))
                                            aria-label=(t!("pagination.go_to", locale = locale, page = number)) {
                                            (number)
                                        }
                                    }
                                }
                            },
                            PaginationItem::Ellipsis => {
                                li class="pagination-ellipsis" aria-hidden="true" { "…" }
                            },
                        }
                    }
                }
                @if page < total_pages {
                    a class="pagination-older" href=(page_url(page + 1, config, category)) {
                        (t!("pagination.older", locale = locale))
                    }
                }
            }
        }
    }
}

fn pagination_items(current: usize, total: usize) -> Vec<PaginationItem> {
    if total == 0 {
        return Vec::new();
    }

    let mut pages = vec![1, total];
    let start = current.saturating_sub(2).max(1);
    let end = current.saturating_add(2).min(total);
    pages.extend(start..=end);
    pages.sort_unstable();
    pages.dedup();

    let mut items = Vec::with_capacity(pages.len() + 2);
    let mut previous = None;
    for page in pages {
        if let Some(previous) = previous {
            match page - previous {
                2 => items.push(PaginationItem::Page(previous + 1)),
                gap if gap > 2 => items.push(PaginationItem::Ellipsis),
                _ => {}
            }
        }
        items.push(PaginationItem::Page(page));
        previous = Some(page);
    }
    items
}

fn page_url(page: usize, config: &BlogConfig, category: Option<&str>) -> String {
    let mut path = if page == 1 {
        "/".into()
    } else {
        format!("/page/{page}")
    };
    if let Some(category) = category {
        path.push_str("?category=");
        path.push_str(category);
    }
    localized_url(&path, &config.default_locale, config)
}

fn document(config: &BlogConfig, title: &str, content: Markup, categories: &[Category]) -> String {
    let favicon_url = config
        .favicon_url
        .clone()
        .unwrap_or_else(|| asset_url("favicon.svg"));
    html! {
        (DOCTYPE)
        html lang=(&config.default_locale) {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                meta name="description" content=(&config.site_description);
                title {
                    (title)
                    @if title != config.site_name { " · " (&config.site_name) }
                }
                link rel="icon" href=(favicon_url);
                link rel="stylesheet" href=(asset_url("blog.min.css"));
                link rel="stylesheet" href=(asset_url("theme-overrides.css"));
            }
            body {
                div class="site-shell" {
                    aside class="site-sidebar" {
                        a class="site-brand" href=(localized_url("/", &config.default_locale, config)) { (&config.site_name) }
                        p class="site-sidebar-description" { (&config.site_description) }
                        nav class="site-navigation" aria-label=(t!("navigation.main", locale = &config.default_locale)) {
                            @for item in &config.navigation {
                                a href=(localized_url(&item.href, &config.default_locale, config)) { (&item.label) }
                            }
                        }
                        (sidebar_categories(config, categories))
                        (search_trigger(config))
                        (language_switcher(
                            config,
                            "sidebar-language-switcher",
                        ))
                    }
                    div class="site-main" {
                        header class="site-header" {
                            a class="site-header-brand" href=(localized_url("/", &config.default_locale, config)) { (&config.site_name) }
                            p { (&config.site_description) }
                            (search_trigger(config))
                            button class="menu-toggle" type="button" data-menu-toggle
                                aria-controls="mobile-menu" aria-expanded="false"
                                aria-label=(t!("menu.toggle", locale = &config.default_locale)) {
                                (menu_icon())
                            }
                            div class="mobile-menu" id="mobile-menu" data-mobile-menu hidden {
                                nav class="site-header-navigation" aria-label=(t!("navigation.main", locale = &config.default_locale)) {
                                    @for item in &config.navigation {
                                        a href=(localized_url(&item.href, &config.default_locale, config)) { (&item.label) }
                                    }
                                }
                                (mobile_categories(config, categories))
                                (language_switcher(
                                    config,
                                    "mobile-language-switcher",
                                ))
                            }
                        }
                        main { (content) }
                        footer class="site-footer" { p { "© " (&config.site_name) } }
                    }
                }
                (search_dialog(config))
                script src=(asset_url("blog.min.js")) defer {}
            }
        }
    }
    .into_string()
}

fn search_trigger(config: &BlogConfig) -> Markup {
    let locale = &config.default_locale;
    html! {
        a class="search-trigger" href=(localized_url("/search", locale, config)) data-search-open
            aria-haspopup="dialog" aria-label=(t!("search.label", locale = locale)) {
            (search_icon())
            span { (t!("search.label", locale = locale)) }
            kbd { "⌘K" }
        }
    }
}

fn search_dialog(config: &BlogConfig) -> Markup {
    let locale = &config.default_locale;
    let action = localized_url("/search", locale, config);
    html! {
        dialog class="search-dialog" id="search-dialog"
            aria-label=(t!("search.label", locale = locale))
            data-search-minimum=(t!("search.minimum", locale = locale))
            data-search-searching=(t!("search.searching", locale = locale))
            data-search-unavailable=(t!("search.unavailable", locale = locale)) {
            div class="search-panel" {
                form class="search-modal-form" action=(action) method="get" {
                    (search_icon())
                    label class="visually-hidden" for="search-modal-input" { (t!("search.description", locale = locale)) }
                    input id="search-modal-input" type="search" name="q"
                        placeholder=(t!("search.description", locale = locale)) autocomplete="off"
                        minlength="2" maxlength="120" required;
                    button class="search-close" type="button" data-search-close
                        aria-label=(t!("search.close", locale = locale)) { "×" }
                }
                div class="search-modal-results" aria-live="polite" {
                    p class="search-state" { (t!("search.minimum", locale = locale)) }
                }
                footer { span { (t!("search.description", locale = locale)) } kbd { "esc" } }
            }
        }
    }
}

fn language_switcher(config: &BlogConfig, class: &str) -> Markup {
    if !config.multilingual_enabled || config.localizations.len() < 2 {
        return html! {};
    }
    let locale = &config.default_locale;
    let active_label = locale_label(locale);
    html! {
        details class={ "language-switcher " (class) }
            aria-label=(t!("language.label", locale = locale)) {
            summary {
                (language_icon())
                span { (active_label) }
            }
            ul class="language-list" {
                @for code in config.locale_codes() {
                    @let label = locale_label(code);
                    @let href = language_url(&config.current_url, code, config);
                    li {
                        @if code.eq_ignore_ascii_case(locale) {
                            a href=(&href) lang=(&code) hreflang=(&code)
                                aria-current="page" { (label) }
                        } @else {
                            a href=(&href) lang=(&code) hreflang=(&code) { (label) }
                        }
                    }
                }
            }
        }
    }
}

fn language_url(current_url: &str, locale: &str, config: &BlogConfig) -> String {
    let path = config
        .language_paths
        .iter()
        .find(|path| path.locale.eq_ignore_ascii_case(locale))
        .map_or(current_url, |path| path.path.as_str());
    localized_url(path, locale, config)
}

fn locale_label(locale: &str) -> String {
    let language = locale.split('-').next().unwrap_or(locale);
    rust_i18n::available_locales!()
        .into_iter()
        .find(|available| {
            available.eq_ignore_ascii_case(locale) || available.eq_ignore_ascii_case(language)
        })
        .map_or_else(
            || locale.to_owned(),
            |available| t!("language.name", locale = available.as_ref()).into_owned(),
        )
}

fn category_url(category: &str, config: &BlogConfig) -> String {
    localized_url(
        &format!("/?category={category}"),
        &config.default_locale,
        config,
    )
}

fn sidebar_categories(config: &BlogConfig, categories: &[Category]) -> Markup {
    html! {
        @if !categories.is_empty() {
            section class="sidebar-categories" {
                h2 { (t!("categories.label", locale = &config.default_locale)) }
                (category_links(config, categories))
            }
        }
    }
}

fn mobile_categories(config: &BlogConfig, categories: &[Category]) -> Markup {
    html! {
        @if !categories.is_empty() {
            details class="mobile-categories" {
                summary {
                    (category_icon())
                    (t!("categories.label", locale = &config.default_locale))
                }
                (category_links(config, categories))
            }
        }
    }
}

fn category_links(config: &BlogConfig, categories: &[Category]) -> Markup {
    html! {
        ul class="category-list" {
            li {
                @if config.active_category.is_none() {
                    a href=(localized_url("/", &config.default_locale, config)) aria-current="page" {
                        span { (t!("categories.all", locale = &config.default_locale)) }
                    }
                } @else {
                    a href=(localized_url("/", &config.default_locale, config)) {
                        span { (t!("categories.all", locale = &config.default_locale)) }
                    }
                }
            }
            @for category in categories {
                li {
                    @let href = category_url(&category.slug, config);
                    @if config.active_category.as_deref() == Some(category.slug.as_str()) {
                        a href=(&href) aria-current="page" {
                            span { (&category.name) }
                            small { (category.count) }
                        }
                    } @else {
                        a href=(&href) {
                            span { (&category.name) }
                            small { (category.count) }
                        }
                    }
                }
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

fn language_icon() -> Markup {
    html! {
        svg class="language-icon" aria-hidden="true" viewBox="0 0 24 24" {
            circle cx="12" cy="12" r="9" {}
            path d="M3 12h18M12 3c2.4 2.5 3.7 5.5 3.7 9S14.4 18.5 12 21M12 3c-2.4 2.5-3.7 5.5-3.7 9s1.3 6.5 3.7 9" {}
        }
    }
}

fn category_icon() -> Markup {
    html! {
        svg class="category-icon" aria-hidden="true" viewBox="0 0 24 24" {
            path d="M4 5.5h6l2 2h8v11H4z" {}
        }
    }
}

fn menu_icon() -> Markup {
    html! {
        svg aria-hidden="true" viewBox="0 0 24 24" {
            path d="M4 7h16M4 12h16M4 17h16" {}
        }
    }
}

fn localized_url(href: &str, locale: &str, config: &BlogConfig) -> String {
    if !href.starts_with('/') || href.starts_with("//") {
        return href.to_owned();
    }

    let (without_fragment, fragment) = href
        .split_once('#')
        .map_or((href, None), |(path, fragment)| (path, Some(fragment)));
    let (path, query) = without_fragment
        .split_once('?')
        .map_or((without_fragment, ""), |(path, query)| (path, query));
    let path = strip_locale_prefix(path, config);
    let mut localized = String::with_capacity(href.len() + locale.len() + 2);
    if locale.eq_ignore_ascii_case(&config.site_default_locale) {
        localized.push_str(path);
    } else {
        localized.push('/');
        localized.push_str(locale);
        if path != "/" {
            localized.push_str(path);
        } else {
            localized.push('/');
        }
    }
    let mut has_parameter = false;
    for parameter in query
        .split('&')
        .filter(|parameter| !parameter.is_empty() && !parameter.starts_with("locale="))
    {
        if !has_parameter {
            localized.push('?');
        } else {
            localized.push('&');
        }
        localized.push_str(parameter);
        has_parameter = true;
    }
    if let Some(fragment) = fragment {
        localized.push('#');
        localized.push_str(fragment);
    }
    localized
}

fn strip_locale_prefix<'a>(path: &'a str, config: &BlogConfig) -> &'a str {
    if !config.multilingual_enabled {
        return path;
    }
    let remainder = path.strip_prefix('/').unwrap_or(path);
    let (first, tail) = remainder
        .split_once('/')
        .map_or((remainder, ""), |(first, tail)| (first, tail));
    if config
        .locale_codes()
        .any(|locale| locale.eq_ignore_ascii_case(first))
    {
        if tail.is_empty() {
            "/"
        } else {
            &path[first.len() + 1..]
        }
    } else {
        path
    }
}

fn asset_url(filename: &str) -> String {
    format!("/p/blog/assets/{filename}?v={ASSET_VERSION}")
}

#[cfg(test)]
mod tests {
    use super::{
        PaginationItem, document, language_switcher, language_url, localized_url, page_url,
        pagination_items,
    };
    use crate::config::{BlogConfig, LanguagePath, SiteLocalization};
    use maud::html;

    #[test]
    fn pagination_keeps_edges_and_current_neighbors() {
        assert_eq!(
            pagination_items(6, 12),
            vec![
                PaginationItem::Page(1),
                PaginationItem::Ellipsis,
                PaginationItem::Page(4),
                PaginationItem::Page(5),
                PaginationItem::Page(6),
                PaginationItem::Page(7),
                PaginationItem::Page(8),
                PaginationItem::Ellipsis,
                PaginationItem::Page(12),
            ]
        );
    }

    #[test]
    fn locale_is_added_to_internal_urls() {
        let config = test_config();
        assert_eq!(localized_url("/", "de", &config), "/de/");
        assert_eq!(localized_url("/about", "de", &config), "/de/about");
        assert_eq!(
            localized_url("/en/search?q=rust&locale=en#results", "de", &config),
            "/de/search?q=rust#results"
        );
        assert_eq!(
            localized_url("https://example.com", "de", &config),
            "https://example.com"
        );
        assert_eq!(localized_url("/de/about", "en", &config), "/about");
    }

    #[test]
    fn pagination_preserves_category_and_locale() {
        let mut config = test_config();
        config.default_locale = "de".into();
        assert_eq!(
            page_url(3, &config, Some("entwicklung")),
            "/de/page/3?category=entwicklung"
        );
    }

    #[test]
    fn language_switch_preserves_the_current_page_and_query() {
        assert_eq!(
            language_url("/article/example?q=rust", "de", &test_config()),
            "/de/article/example?q=rust"
        );
        assert_eq!(
            language_url("/de/article/beispiel?q=rust", "en", &test_config()),
            "/article/beispiel?q=rust"
        );
    }

    #[test]
    fn language_switch_uses_the_translated_content_slug() {
        let mut config = test_config();
        config.site_default_locale = "de".into();
        config.default_locale = "en".into();
        config.language_paths.push(LanguagePath {
            locale: "de".into(),
            path: "/ueber-uns".into(),
        });

        assert_eq!(language_url("/en/about", "de", &config), "/ueber-uns");
    }

    #[test]
    fn language_options_come_from_site_localizations() {
        let config = test_config();
        assert_eq!(config.locale_codes().collect::<Vec<_>>(), ["en", "de"]);
    }

    #[test]
    fn language_selector_is_hidden_when_multilingual_support_is_disabled() {
        let mut config = test_config();
        config.multilingual_enabled = false;

        assert!(language_switcher(&config, "test").into_string().is_empty());
    }

    #[test]
    fn document_title_does_not_repeat_the_site_name() {
        let config = test_config();

        let home = document(&config, "Notes", html! {}, &[]);
        assert!(home.contains("<title>Notes</title>"));
        assert!(!home.contains("Notes · Notes"));

        let article = document(&config, "An article", html! {}, &[]);
        assert!(article.contains("<title>An article · Notes</title>"));
    }

    fn test_config() -> BlogConfig {
        BlogConfig {
            site_name: "Notes".into(),
            site_description: String::new(),
            default_locale: "en".into(),
            multilingual_enabled: true,
            favicon_url: None,
            article_type: "article".into(),
            page_type: "page".into(),
            index_page_slug: None,
            posts_per_page: 6,
            localizations: vec![
                SiteLocalization {
                    locale: "en".into(),
                    site_name: "Notes".into(),
                    site_description: String::new(),
                    index_page_slug: None,
                    navigation: Vec::new(),
                },
                SiteLocalization {
                    locale: "de".into(),
                    site_name: "Notizen".into(),
                    site_description: String::new(),
                    index_page_slug: None,
                    navigation: Vec::new(),
                },
            ],
            navigation: Vec::new(),
            active_category: None,
            current_url: "/".into(),
            site_default_locale: "en".into(),
            language_paths: Vec::new(),
        }
    }
}
