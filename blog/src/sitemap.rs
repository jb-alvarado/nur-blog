use crate::{
    bindings::{
        exports::nur::cms::http_handler::{PluginError, Request, Response},
        nur::cms::{configuration, types::Header},
    },
    config::BlogConfig,
    content::{SitemapEntry, sitemap_entries},
};
use maud::html;

const ASSET_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn response(request: &Request, config: &BlogConfig) -> Result<Response, PluginError> {
    let origin = preferred_origin(configuration::public_url(), &request.headers)?;
    let locales = sitemap_locales(config);
    let mut entries = sitemap_entries(&config.article_type, &config.page_type, &locales)?;
    exclude_index_pages(&mut entries, config);
    let body = render(&origin, config, &locales, &entries);

    Ok(Response {
        status: 200,
        headers: vec![
            Header {
                name: "content-type".into(),
                value: "application/xml; charset=utf-8".into(),
            },
            Header {
                name: "cache-control".into(),
                value: "public, max-age=3600".into(),
            },
        ],
        body: body.into_bytes(),
    })
}

fn exclude_index_pages(entries: &mut Vec<SitemapEntry>, config: &BlogConfig) {
    entries.retain(|entry| {
        entry.article
            || config
                .localizations
                .iter()
                .find(|localization| localization.locale.eq_ignore_ascii_case(&entry.locale))
                .and_then(|localization| localization.index_page_slug.as_deref())
                != Some(entry.slug.as_str())
    });
}

fn render(
    origin: &str,
    config: &BlogConfig,
    locales: &[String],
    entries: &[SitemapEntry],
) -> String {
    let markup = html! {
        urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" {
            @for locale in locales {
                url {
                    loc {
                        (origin)
                        "/"
                        @if !locale.eq_ignore_ascii_case(&config.site_default_locale) { (locale) "/" }
                    }
                }
            }
            @for entry in entries {
                url {
                    loc {
                        (origin)
                        "/"
                        @if {
                            !entry
                                .locale
                                .eq_ignore_ascii_case(&config.site_default_locale)
                        } { (&entry.locale) "/" }
                        @if entry.article { (&config.article_type) "/" }
                        (&entry.slug)
                    }
                    @if let Some(updated_at) = &entry.updated_at {
                        lastmod { (updated_at) }
                    }
                }
            }
        }
    };
    let document = markup.into_string();
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<?xml-stylesheet type=\"text/xsl\" href=\"/p/blog/assets/sitemap.xsl?v={ASSET_VERSION}\"?>\n{document}"
    )
}

fn sitemap_locales(config: &BlogConfig) -> Vec<String> {
    let mut locales = vec![config.site_default_locale.clone()];
    if config.multilingual_enabled {
        locales.extend(
            config
                .locale_codes()
                .filter(|locale| !locale.eq_ignore_ascii_case(&config.site_default_locale))
                .map(str::to_owned),
        );
    }
    locales
}

pub(crate) fn preferred_origin(
    configured_public_url: Option<String>,
    headers: &[Header],
) -> Result<String, PluginError> {
    configured_public_url.map_or_else(|| request_origin(headers), Ok)
}

fn request_origin(headers: &[Header]) -> Result<String, PluginError> {
    let forwarded = header(headers, "forwarded").and_then(|value| value.split(',').next());
    let scheme = forwarded
        .and_then(|value| forwarded_value(value, "proto"))
        .or_else(|| first_header_value(headers, "x-forwarded-proto"))
        .unwrap_or("http");
    let host = forwarded
        .and_then(|value| forwarded_value(value, "host"))
        .or_else(|| first_header_value(headers, "x-forwarded-host"))
        .or_else(|| header(headers, "host"))
        .ok_or_else(|| PluginError::Failed("request host is unavailable".into()))?;

    let scheme = if scheme.eq_ignore_ascii_case("https") {
        "https"
    } else if scheme.eq_ignore_ascii_case("http") {
        "http"
    } else {
        return Err(PluginError::Failed("request origin is invalid".into()));
    };
    if !valid_host(host) {
        return Err(PluginError::Failed("request origin is invalid".into()));
    }
    Ok(format!("{scheme}://{host}"))
}

fn header<'a>(headers: &'a [Header], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|header| header.name.eq_ignore_ascii_case(name))
        .map(|header| header.value.trim())
        .filter(|value| !value.is_empty())
}

fn first_header_value<'a>(headers: &'a [Header], name: &str) -> Option<&'a str> {
    header(headers, name)
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn forwarded_value<'a>(forwarded: &'a str, name: &str) -> Option<&'a str> {
    forwarded.split(';').find_map(|parameter| {
        let (key, value) = parameter.trim().split_once('=')?;
        key.eq_ignore_ascii_case(name)
            .then(|| value.trim().trim_matches('"'))
            .filter(|value| !value.is_empty())
    })
}

fn valid_host(host: &str) -> bool {
    !host.is_empty()
        && host.len() <= 255
        && host.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b':' | b'[' | b']')
        })
}

#[cfg(test)]
mod tests {
    use super::{exclude_index_pages, preferred_origin, render, request_origin, sitemap_locales};
    use crate::{
        bindings::nur::cms::types::Header,
        config::{BlogConfig, SiteLocalization},
        content::SitemapEntry,
    };

    #[test]
    fn uses_forwarded_public_origin() {
        let headers = vec![
            Header {
                name: "host".into(),
                value: "127.0.0.1:8777".into(),
            },
            Header {
                name: "forwarded".into(),
                value: "for=192.0.2.1;proto=https;host=blog.example.org".into(),
            },
        ];

        assert_eq!(
            request_origin(&headers).unwrap(),
            "https://blog.example.org"
        );
    }

    #[test]
    fn configured_public_url_has_priority_over_request_headers() {
        let headers = vec![Header {
            name: "host".into(),
            value: "internal.example.org:8777".into(),
        }];

        assert_eq!(
            preferred_origin(Some("https://blog.example.org".into()), &headers).unwrap(),
            "https://blog.example.org"
        );
    }

    #[test]
    fn renders_home_pages_articles_and_last_modified_dates() {
        let config = test_config();
        let entries = vec![
            SitemapEntry {
                slug: "about".into(),
                updated_at: None,
                article: false,
                locale: "en".into(),
            },
            SitemapEntry {
                slug: "rust-and-wasm".into(),
                updated_at: Some("2026-09-22T10:00:00Z".into()),
                article: true,
                locale: "de".into(),
            },
        ];

        let xml = render(
            "https://example.org",
            &config,
            &["en".into(), "de".into()],
            &entries,
        );
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains("<loc>https://example.org/</loc>"));
        assert!(xml.contains("<loc>https://example.org/de/</loc>"));
        assert!(xml.contains("<loc>https://example.org/about</loc>"));
        assert!(xml.contains("<loc>https://example.org/de/article/rust-and-wasm</loc>"));
        assert!(xml.contains("<lastmod>2026-09-22T10:00:00Z</lastmod>"));
        assert!(xml.contains("sitemap.xsl?v=0.1.0"));
    }

    #[test]
    fn sitemap_locales_start_with_the_configured_default() {
        let mut config = test_config();
        config.site_default_locale = "de".into();
        let locales = sitemap_locales(&config);
        assert_eq!(locales.first().map(String::as_str), Some("de"));
        assert!(locales.iter().any(|locale| locale == "en"));
        assert!(locales.iter().any(|locale| locale == "de"));
    }

    #[test]
    fn sitemap_uses_only_the_default_locale_when_multilingual_support_is_disabled() {
        let mut config = test_config();
        config.multilingual_enabled = false;

        assert_eq!(sitemap_locales(&config), ["en"]);
    }

    #[test]
    fn excludes_only_the_index_page() {
        let mut entries = vec![
            SitemapEntry {
                slug: "index".into(),
                updated_at: None,
                article: false,
                locale: "en".into(),
            },
            SitemapEntry {
                slug: "index".into(),
                updated_at: None,
                article: true,
                locale: "en".into(),
            },
            SitemapEntry {
                slug: "about".into(),
                updated_at: None,
                article: false,
                locale: "en".into(),
            },
        ];

        exclude_index_pages(&mut entries, &test_config());

        assert_eq!(entries.len(), 2);
        assert!(
            entries
                .iter()
                .any(|entry| entry.article && entry.slug == "index")
        );
        assert!(
            entries
                .iter()
                .any(|entry| !entry.article && entry.slug == "about")
        );
    }

    #[test]
    fn rejects_an_unsafe_host_header() {
        let headers = vec![Header {
            name: "host".into(),
            value: "example.org/path".into(),
        }];
        assert!(request_origin(&headers).is_err());
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
            index_page_slug: Some("index".into()),
            posts_per_page: 6,
            localizations: vec![
                SiteLocalization {
                    locale: "en".into(),
                    site_name: "Notes".into(),
                    site_description: String::new(),
                    index_page_slug: Some("index".into()),
                    navigation: Vec::new(),
                },
                SiteLocalization {
                    locale: "de".into(),
                    site_name: "Notizen".into(),
                    site_description: String::new(),
                    index_page_slug: Some("start".into()),
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
