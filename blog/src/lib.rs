use bindings::{
    exports::nur::cms::http_handler::{Guest, PluginError, Request, Response},
    nur::cms::types::Header,
};
use percent_encoding::percent_decode_str;
use rust_i18n::t;

mod config;
mod content;
mod db;
mod robots;
mod sitemap;
mod view;

mod bindings {
    wit_bindgen::generate!({
        path: "../../nur-cms/backend/plugins/wit/nur-cms-plugin",
        world: "cms-plugin",
    });
}

use crate::{
    config::{BlogConfig, LanguagePath, valid_slug},
    content::{Entry, entry},
    db::handles::configuration,
    view::{article_page, content_page, home_page, response, search_page, search_results},
};

rust_i18n::i18n!("locales", fallback = "en");

struct Blog;

impl Guest for Blog {
    fn handle(request: Request) -> Result<Response, PluginError> {
        match request.route_id.as_str() {
            "settings" => settings_response(configuration::load_config()?),
            "settings-update" => update_settings(request.body),
            "home" => render_home(&request, 0, None),
            "pagination" => render_pagination(&request, None, "number"),
            "search" => render_search(&request, None),
            "favicon" => Ok(default_favicon_response()),
            "sitemap" => render_sitemap(&request),
            "robots" => robots::response(&request),
            "localized-home" => render_one_segment(&request),
            "one-segment" => render_one_segment(&request),
            "two-segments" => render_two_segments(&request),
            "three-segments" => render_three_segments(&request),
            _ => Err(PluginError::NotFound),
        }
    }
}

fn default_favicon_response() -> Response {
    Response {
        status: 200,
        headers: vec![
            Header {
                name: "content-type".into(),
                value: "image/svg+xml".into(),
            },
            Header {
                name: "cache-control".into(),
                value: "public, max-age=86400".into(),
            },
        ],
        body: include_bytes!("../assets/favicon.svg").to_vec(),
    }
}

fn render_home(
    request: &Request,
    offset: usize,
    locale: Option<&str>,
) -> Result<Response, PluginError> {
    let config = public_config(request, locale)?;
    home_response(config, offset)
}

fn home_response(config: BlogConfig, offset: usize) -> Result<Response, PluginError> {
    response(
        &config,
        config.site_name.clone(),
        home_page(&config, offset)?,
    )
}

fn render_pagination(
    request: &Request,
    locale: Option<&str>,
    number_parameter: &str,
) -> Result<Response, PluginError> {
    let page = path_param(request, number_parameter).and_then(parse_page_number)?;
    let config = public_config(request, locale)?;
    pagination_response(config, page)
}

fn pagination_response(config: BlogConfig, page: usize) -> Result<Response, PluginError> {
    response(
        &config,
        t!(
            "pagination.title",
            locale = &config.default_locale,
            page = page
        )
        .to_string(),
        home_page(&config, (page - 1) * config.posts_per_page)?,
    )
}

fn article_response(
    mut config: BlogConfig,
    article_type: &str,
    slug: &str,
) -> Result<Response, PluginError> {
    if article_type != config.article_type || !valid_slug(slug) {
        return Err(PluginError::NotFound);
    }

    let article =
        entry(&config.article_type, &config.default_locale, slug)?.ok_or(PluginError::NotFound)?;
    config.language_paths = content_language_paths(&config, &article, true)?;
    let title = article
        .title(&t!("content.article", locale = &config.default_locale))
        .to_owned();
    response(
        &config,
        title,
        article_page(&article, &config.default_locale),
    )
}

fn render_search(request: &Request, locale: Option<&str>) -> Result<Response, PluginError> {
    let config = public_config(request, locale)?;
    search_response(request, config)
}

fn search_response(request: &Request, config: BlogConfig) -> Result<Response, PluginError> {
    let query = query_parameter(request.query.as_deref(), "q")?
        .unwrap_or_default()
        .trim()
        .to_owned();
    if query.chars().count() > 120 {
        return Err(PluginError::BadRequest(
            "search query must contain at most 120 characters".into(),
        ));
    }
    let results = if query.chars().count() >= 2 {
        content::search(
            &config.article_type,
            &config.page_type,
            &config.default_locale,
            &query,
        )?
    } else {
        Vec::new()
    };

    if query_parameter(request.query.as_deref(), "fragment")?.as_deref() == Some("1") {
        return Ok(Response {
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
            body: search_results(&query, &results, &config)
                .into_string()
                .into_bytes(),
        });
    }

    response(
        &config,
        t!("search.label", locale = &config.default_locale).to_string(),
        search_page(&query, &results, &config),
    )
}

fn content_page_response(mut config: BlogConfig, slug: &str) -> Result<Response, PluginError> {
    if !valid_slug(slug) {
        return Err(PluginError::NotFound);
    }

    let page =
        entry(&config.page_type, &config.default_locale, slug)?.ok_or(PluginError::NotFound)?;
    config.language_paths = content_language_paths(&config, &page, false)?;
    let title = page
        .title(&t!("content.page", locale = &config.default_locale))
        .to_owned();
    response(&config, title, content_page(&page, &config.default_locale))
}

fn content_language_paths(
    config: &BlogConfig,
    entry: &Entry,
    article: bool,
) -> Result<Vec<LanguagePath>, PluginError> {
    if !config.multilingual_enabled {
        return Ok(Vec::new());
    }
    let locales = config.locale_codes().map(str::to_owned).collect::<Vec<_>>();
    let content_type = if article {
        &config.article_type
    } else {
        &config.page_type
    };
    let translations = match entry.group_id {
        Some(group_id) => content::translated_entries(content_type, &locales, group_id)?,
        None => Vec::new(),
    };
    let mut paths = Vec::with_capacity(locales.len().saturating_sub(1));
    for locale in locales {
        if locale.eq_ignore_ascii_case(&config.default_locale) {
            continue;
        }
        let translated_slug = translations
            .iter()
            .find(|entry| entry.locale.eq_ignore_ascii_case(&locale))
            .map(|entry| entry.slug.as_str());
        let path = translated_slug.map_or_else(
            || "/".into(),
            |slug| {
                if article {
                    format!("/{}/{slug}", config.article_type)
                } else {
                    format!("/{slug}")
                }
            },
        );
        paths.push(LanguagePath { locale, path });
    }
    Ok(paths)
}

fn render_sitemap(request: &Request) -> Result<Response, PluginError> {
    let config = configuration::load_config()?;
    sitemap::response(request, &config)
}

fn render_one_segment(request: &Request) -> Result<Response, PluginError> {
    let first = path_param(request, "first")?;
    let config = configuration::load_config()?;
    let default_locale = config.default_locale.clone();
    if let Some(locale) = available_locale(first, &config) {
        if locale.eq_ignore_ascii_case(&default_locale) {
            return Ok(redirect(request, "/"));
        }
        if !request.path.ends_with('/') {
            return Ok(redirect(request, &format!("/{locale}/")));
        }
        home_response(prepare_public_config(config, request, Some(&locale))?, 0)
    } else {
        content_page_response(prepare_public_config(config, request, None)?, first)
    }
}

fn render_two_segments(request: &Request) -> Result<Response, PluginError> {
    let first = path_param(request, "first")?;
    let second = path_param(request, "second")?;
    let config = configuration::load_config()?;
    let default_locale = config.default_locale.clone();
    if let Some(locale) = available_locale(first, &config) {
        if locale.eq_ignore_ascii_case(&default_locale) {
            return Ok(redirect(request, &format!("/{second}")));
        }
        if second == "search" {
            search_response(
                request,
                prepare_public_config(config, request, Some(&locale))?,
            )
        } else {
            content_page_response(
                prepare_public_config(config, request, Some(&locale))?,
                second,
            )
        }
    } else {
        article_response(prepare_public_config(config, request, None)?, first, second)
    }
}

fn render_three_segments(request: &Request) -> Result<Response, PluginError> {
    let first = path_param(request, "first")?;
    let second = path_param(request, "second")?;
    let third = path_param(request, "third")?;
    let config = configuration::load_config()?;
    let default_locale = config.default_locale.clone();
    let locale = available_locale(first, &config).ok_or(PluginError::NotFound)?;
    if locale.eq_ignore_ascii_case(&default_locale) {
        return Ok(redirect(request, &format!("/{second}/{third}")));
    }
    if second == "page" {
        let page = parse_page_number(third)?;
        pagination_response(prepare_public_config(config, request, Some(&locale))?, page)
    } else {
        article_response(
            prepare_public_config(config, request, Some(&locale))?,
            second,
            third,
        )
    }
}

fn available_locale(candidate: &str, config: &BlogConfig) -> Option<String> {
    if !config.multilingual_enabled {
        return None;
    }
    config
        .locale_codes()
        .find(|locale| locale.eq_ignore_ascii_case(candidate))
        .map(str::to_owned)
}

fn redirect(request: &Request, path: &str) -> Response {
    let query = request
        .query
        .as_deref()
        .into_iter()
        .flat_map(|query| query.split('&'))
        .filter(|parameter| !parameter.is_empty() && !parameter.starts_with("locale="))
        .collect::<Vec<_>>()
        .join("&");
    let location = if query.is_empty() {
        path.to_owned()
    } else {
        format!("{path}?{query}")
    };
    Response {
        status: 308,
        headers: vec![Header {
            name: "location".into(),
            value: location,
        }],
        body: Vec::new(),
    }
}

fn path_param<'a>(request: &'a Request, name: &str) -> Result<&'a str, PluginError> {
    request
        .path_params
        .iter()
        .find(|parameter| parameter.name == name)
        .map(|parameter| parameter.value.as_str())
        .ok_or(PluginError::NotFound)
}

fn public_config(request: &Request, locale: Option<&str>) -> Result<BlogConfig, PluginError> {
    prepare_public_config(configuration::load_config()?, request, locale)
}

fn prepare_public_config(
    mut config: BlogConfig,
    request: &Request,
    locale: Option<&str>,
) -> Result<BlogConfig, PluginError> {
    config.current_url = request_url(request);
    let active_locale = locale.unwrap_or(&config.site_default_locale).to_owned();
    config
        .activate_locale(&active_locale)
        .map_err(|_| PluginError::NotFound)?;
    if let Some(category) = query_parameter(request.query.as_deref(), "category")? {
        if !valid_slug(&category) {
            return Err(PluginError::BadRequest("category is invalid".into()));
        }
        config.active_category = Some(category);
    }
    Ok(config)
}

fn request_url(request: &Request) -> String {
    request
        .query
        .as_deref()
        .filter(|query| !query.is_empty())
        .map_or_else(
            || request.path.clone(),
            |query| format!("{}?{query}", request.path),
        )
}

fn parse_page_number(value: &str) -> Result<usize, PluginError> {
    value
        .parse::<usize>()
        .ok()
        .filter(|number| *number > 0 && *number <= 166_667)
        .ok_or(PluginError::NotFound)
}

fn query_parameter(query: Option<&str>, name: &str) -> Result<Option<String>, PluginError> {
    let Some(value) = query.and_then(|query| {
        query.split('&').find_map(|parameter| {
            let (key, value) = parameter.split_once('=').unwrap_or((parameter, ""));
            (key == name).then_some(value)
        })
    }) else {
        return Ok(None);
    };
    let value = value.replace('+', " ");
    percent_decode_str(&value)
        .decode_utf8()
        .map(|value| Some(value.into_owned()))
        .map_err(|_| PluginError::BadRequest("query string is not valid UTF-8".into()))
}

fn update_settings(body: Vec<u8>) -> Result<Response, PluginError> {
    let config: BlogConfig = serde_json::from_slice(&body)
        .map_err(|_| PluginError::BadRequest("settings must be valid JSON".into()))?;
    configuration::update(config)?;

    Ok(Response {
        status: 204,
        headers: Vec::new(),
        body: Vec::new(),
    })
}

fn settings_response(config: BlogConfig) -> Result<Response, PluginError> {
    let body = serde_json::to_vec(&config)
        .map_err(|_| PluginError::Failed("settings could not be encoded".into()))?;
    Ok(Response {
        status: 200,
        headers: vec![Header {
            name: "content-type".into(),
            value: "application/json; charset=utf-8".into(),
        }],
        body,
    })
}

bindings::export!(Blog with_types_in bindings);

#[cfg(test)]
mod tests {
    use super::{available_locale, query_parameter, redirect};
    use crate::bindings::exports::nur::cms::http_handler::Request;
    use crate::config::{BlogConfig, SiteLocalization};

    #[test]
    fn decodes_search_query_parameter() {
        assert_eq!(
            query_parameter(Some("q=rust+und+wasm%3F&fragment=1"), "q").unwrap(),
            Some("rust und wasm?".into())
        );
    }

    #[test]
    fn regional_locale_uses_base_language_translation() {
        assert_eq!(
            rust_i18n::t!("article.read", locale = "de-DE").to_string(),
            "Artikel lesen"
        );
        assert_eq!(
            rust_i18n::t!("pagination.title", locale = "de", page = 2).to_string(),
            "Seite 2"
        );
    }

    #[test]
    fn recognizes_compiled_and_configured_locales() {
        let config = locale_config();
        assert_eq!(available_locale("de", &config).as_deref(), Some("de"));
        assert_eq!(available_locale("en", &config).as_deref(), Some("en"));
        assert!(available_locale("fr", &config).is_none());
    }

    #[test]
    fn ignores_locale_routes_when_multilingual_support_is_disabled() {
        let mut config = locale_config();
        config.multilingual_enabled = false;

        assert!(available_locale("de", &config).is_none());
    }

    #[test]
    fn canonical_redirect_keeps_query_without_legacy_locale() {
        let request = Request {
            route_id: "two-segments".into(),
            method: "GET".into(),
            path: "/en/search".into(),
            path_params: Vec::new(),
            query: Some("q=rust&locale=en".into()),
            headers: Vec::new(),
            body: Vec::new(),
            identity: None,
        };
        let response = redirect(&request, "/search");

        assert_eq!(response.status, 308);
        assert_eq!(response.headers[0].value, "/search?q=rust");
    }

    fn locale_config() -> BlogConfig {
        BlogConfig {
            default_locale: "en".into(),
            multilingual_enabled: true,
            favicon_url: None,
            article_type: "article".into(),
            page_type: "page".into(),
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
            site_name: "Notes".into(),
            site_description: String::new(),
            index_page_slug: None,
            navigation: Vec::new(),
            active_category: None,
            current_url: "/".into(),
            site_default_locale: "en".into(),
            language_paths: Vec::new(),
        }
    }
}
