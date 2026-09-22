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
            &config.default_locale,
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

pub fn search_page(query: &str, results: &[SearchResult], locale: &str) -> Markup {
    html! {
        section class="search-page" {
            p class="eyebrow" { (t!("search.label", locale = locale)) }
            h1 { (t!("search.title", locale = locale)) }
            form class="search-page-form" action="/search" method="get" {
                input type="hidden" name="locale" value=(locale);
                label class="visually-hidden" for="search-page-input" { (t!("search.description", locale = locale)) }
                input id="search-page-input" type="search" name="q" value=(query)
                    placeholder=(t!("search.description", locale = locale)) minlength="2" maxlength="120" required;
                button type="submit" { (t!("search.label", locale = locale)) }
            }
            (search_results(query, results, locale))
        }
    }
}

pub fn search_results(query: &str, results: &[SearchResult], locale: &str) -> Markup {
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
                            a href=(localized_url(&result.href, locale)) {
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

fn pagination(page: usize, total_pages: usize, locale: &str, category: Option<&str>) -> Markup {
    let items = pagination_items(page, total_pages);
    html! {
        @if total_pages > 1 {
            nav class="pagination" aria-label=(t!("pagination.label", locale = locale)) {
                @if page > 1 {
                    a class="pagination-newer" href=(page_url(page - 1, locale, category)) {
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
                                        a href=(page_url(number, locale, category))
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
                    a class="pagination-older" href=(page_url(page + 1, locale, category)) {
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

fn page_url(page: usize, locale: &str, category: Option<&str>) -> String {
    let mut path = if page == 1 {
        "/".into()
    } else {
        format!("/page/{page}")
    };
    if let Some(category) = category {
        path.push_str("?category=");
        path.push_str(category);
    }
    localized_url(&path, locale)
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
                        a class="site-brand" href=(localized_url("/", &config.default_locale)) { (&config.site_name) }
                        p class="site-sidebar-description" { (&config.site_description) }
                        nav class="site-navigation" aria-label=(t!("navigation.main", locale = &config.default_locale)) {
                            @for item in &config.navigation {
                                a href=(localized_url(&item.href, &config.default_locale)) { (&item.label) }
                            }
                        }
                        (sidebar_categories(config, categories))
                        (search_trigger(&config.default_locale))
                        (language_switcher(
                            config,
                            "sidebar-language-switcher",
                        ))
                    }
                    div class="site-main" {
                        header class="site-header" {
                            a class="site-header-brand" href=(localized_url("/", &config.default_locale)) { (&config.site_name) }
                            p { (&config.site_description) }
                            (search_trigger(&config.default_locale))
                            button class="menu-toggle" type="button" data-menu-toggle
                                aria-controls="mobile-menu" aria-expanded="false"
                                aria-label=(t!("menu.toggle", locale = &config.default_locale)) {
                                (menu_icon())
                            }
                            div class="mobile-menu" id="mobile-menu" data-mobile-menu hidden {
                                nav class="site-header-navigation" aria-label=(t!("navigation.main", locale = &config.default_locale)) {
                                    @for item in &config.navigation {
                                        a href=(localized_url(&item.href, &config.default_locale)) { (&item.label) }
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
                (search_dialog(&config.default_locale))
                script src=(asset_url("blog.min.js")) defer {}
            }
        }
    }
    .into_string()
}

fn search_trigger(locale: &str) -> Markup {
    html! {
        a class="search-trigger" href=(localized_url("/search", locale)) data-search-open
            aria-haspopup="dialog" aria-label=(t!("search.label", locale = locale)) {
            (search_icon())
            span { (t!("search.label", locale = locale)) }
            kbd { "⌘K" }
        }
    }
}

fn search_dialog(locale: &str) -> Markup {
    html! {
        dialog class="search-dialog" id="search-dialog"
            aria-label=(t!("search.label", locale = locale))
            data-search-minimum=(t!("search.minimum", locale = locale))
            data-search-searching=(t!("search.searching", locale = locale))
            data-search-unavailable=(t!("search.unavailable", locale = locale)) {
            div class="search-panel" {
                form class="search-modal-form" action="/search" method="get" {
                    (search_icon())
                    input type="hidden" name="locale" value=(locale);
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
    let locale = &config.default_locale;
    let active_language = locale.split('-').next().unwrap_or(locale);
    let options = rust_i18n::available_locales!();
    let active_option = options.iter().find(|option| {
        option.eq_ignore_ascii_case(locale) || option.eq_ignore_ascii_case(active_language)
    });
    let active_label = active_option.map_or_else(
        || locale.as_str().into(),
        |code| t!("language.name", locale = code.as_ref()),
    );
    html! {
        details class={ "language-switcher " (class) }
            aria-label=(t!("language.label", locale = locale)) {
            summary {
                (language_icon())
                span { (active_label) }
            }
            ul class="language-list" {
                @for code in &options {
                    @let label = t!("language.name", locale = code.as_ref());
                    @let href = language_url(&config.current_url, code.as_ref());
                    li {
                        @if code.eq_ignore_ascii_case(locale) || code.eq_ignore_ascii_case(active_language) {
                            a href=(&href) lang=(&code) hreflang=(&code)
                                aria-current="page" { (label) }
                        } @else {
                            a href=(&href) lang=(&code) hreflang=(&code) { (label) }
                        }
                    }
                }
                @if active_option.is_none() {
                    @let href = language_url(&config.current_url, locale);
                    li {
                        a href=(&href) lang=(locale) hreflang=(locale)
                            aria-current="page" { (locale) }
                    }
                }
            }
        }
    }
}

fn language_url(current_url: &str, locale: &str) -> String {
    localized_url(current_url, locale)
}

fn category_url(category: &str, locale: &str) -> String {
    localized_url(&format!("/?category={category}"), locale)
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
                    a href=(localized_url("/", &config.default_locale)) aria-current="page" {
                        span { (t!("categories.all", locale = &config.default_locale)) }
                    }
                } @else {
                    a href=(localized_url("/", &config.default_locale)) {
                        span { (t!("categories.all", locale = &config.default_locale)) }
                    }
                }
            }
            @for category in categories {
                li {
                    @let href = category_url(&category.slug, &config.default_locale);
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

fn localized_url(href: &str, locale: &str) -> String {
    if !href.starts_with('/') || href.starts_with("//") {
        return href.to_owned();
    }

    let (without_fragment, fragment) = href
        .split_once('#')
        .map_or((href, None), |(path, fragment)| (path, Some(fragment)));
    let (path, query) = without_fragment
        .split_once('?')
        .map_or((without_fragment, ""), |(path, query)| (path, query));
    let mut localized = String::with_capacity(href.len() + locale.len() + 8);
    localized.push_str(path);
    localized.push('?');
    let mut has_parameter = false;
    for parameter in query
        .split('&')
        .filter(|parameter| !parameter.is_empty() && !parameter.starts_with("locale="))
    {
        if has_parameter {
            localized.push('&');
        }
        localized.push_str(parameter);
        has_parameter = true;
    }
    if has_parameter {
        localized.push('&');
    }
    localized.push_str("locale=");
    localized.push_str(locale);
    if let Some(fragment) = fragment {
        localized.push('#');
        localized.push_str(fragment);
    }
    localized
}

fn asset_url(filename: &str) -> String {
    format!("/p/blog/assets/{filename}?v={ASSET_VERSION}")
}

#[cfg(test)]
mod tests {
    use super::{
        PaginationItem, document, language_url, localized_url, page_url, pagination_items,
    };
    use crate::config::BlogConfig;
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
        assert_eq!(localized_url("/about", "de"), "/about?locale=de");
        assert_eq!(
            localized_url("/search?q=rust&locale=en#results", "de"),
            "/search?q=rust&locale=de#results"
        );
        assert_eq!(
            localized_url("https://example.com", "de"),
            "https://example.com"
        );
    }

    #[test]
    fn pagination_preserves_category_and_locale() {
        assert_eq!(
            page_url(3, "de", Some("entwicklung")),
            "/page/3?category=entwicklung&locale=de"
        );
    }

    #[test]
    fn language_switch_preserves_the_current_page_and_query() {
        assert_eq!(
            language_url("/article/example?q=rust&locale=en", "de"),
            "/article/example?q=rust&locale=de"
        );
    }

    #[test]
    fn language_options_come_from_compiled_locales() {
        let locales = rust_i18n::available_locales!();
        assert!(locales.iter().any(|locale| locale == "de"));
        assert!(locales.iter().any(|locale| locale == "en"));
    }

    #[test]
    fn document_title_does_not_repeat_the_site_name() {
        let config = BlogConfig {
            site_name: "Notes".into(),
            site_description: String::new(),
            default_locale: "en".into(),
            favicon_url: None,
            article_type: "article".into(),
            page_type: "page".into(),
            index_page_slug: None,
            posts_per_page: 6,
            navigation: Vec::new(),
            active_category: None,
            current_url: "/".into(),
        };

        let home = document(&config, "Notes", html! {}, &[]);
        assert!(home.contains("<title>Notes</title>"));
        assert!(!home.contains("Notes · Notes"));

        let article = document(&config, "An article", html! {}, &[]);
        assert!(article.contains("<title>An article · Notes</title>"));
    }
}
