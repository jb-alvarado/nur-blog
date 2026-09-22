use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlogConfig {
    #[serde(default = "default_locale")]
    pub default_locale: String,
    #[serde(default = "enabled")]
    pub multilingual_enabled: bool,
    #[serde(default)]
    pub favicon_url: Option<String>,
    pub article_type: String,
    pub page_type: String,
    pub posts_per_page: usize,
    #[serde(default)]
    pub localizations: Vec<SiteLocalization>,
    #[serde(skip)]
    pub site_name: String,
    #[serde(skip)]
    pub site_description: String,
    #[serde(skip)]
    pub index_page_slug: Option<String>,
    #[serde(skip)]
    pub navigation: Vec<NavigationItem>,
    #[serde(skip)]
    pub active_category: Option<String>,
    #[serde(skip)]
    pub current_url: String,
    #[serde(skip)]
    pub site_default_locale: String,
    #[serde(skip)]
    pub language_paths: Vec<LanguagePath>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SiteLocalization {
    pub locale: String,
    pub site_name: String,
    #[serde(default)]
    pub site_description: String,
    #[serde(default)]
    pub index_page_slug: Option<String>,
    #[serde(default)]
    pub navigation: Vec<NavigationItem>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NavigationItem {
    pub label: String,
    pub href: String,
}

#[derive(Clone, Debug)]
pub struct LanguagePath {
    pub locale: String,
    pub path: String,
}

impl BlogConfig {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !valid_locale(&self.default_locale) {
            return Err("default locale is invalid");
        }
        if self.localizations.is_empty() || self.localizations.len() > 20 {
            return Err("one to twenty localizations are required");
        }
        if !self
            .localizations
            .iter()
            .any(|item| item.locale.eq_ignore_ascii_case(&self.default_locale))
        {
            return Err("default locale requires a localization");
        }
        for (index, localization) in self.localizations.iter().enumerate() {
            if !valid_locale(&localization.locale)
                || self.localizations[..index]
                    .iter()
                    .any(|item| item.locale.eq_ignore_ascii_case(&localization.locale))
            {
                return Err("localizations contain an invalid or duplicate locale");
            }
            if localization.site_name.trim().is_empty()
                || localization.site_name.chars().count() > 120
            {
                return Err("site name must contain at most 120 characters");
            }
            if localization.site_description.chars().count() > 300 {
                return Err("site description must contain at most 300 characters");
            }
            if localization
                .index_page_slug
                .as_deref()
                .is_some_and(|slug| !valid_slug(slug))
            {
                return Err("index page slug is invalid");
            }
            if localization.navigation.len() > 20 {
                return Err("navigation may contain at most 20 links per locale");
            }
            if localization.navigation.iter().any(|item| {
                item.label.trim().is_empty()
                    || item.label.chars().count() > 80
                    || !valid_href(&item.href)
            }) {
                return Err("navigation contains an invalid link");
            }
        }
        if self
            .favicon_url
            .as_deref()
            .is_some_and(|url| !valid_href(url))
        {
            return Err("favicon URL is invalid");
        }
        if !valid_slug(&self.article_type) || !valid_slug(&self.page_type) {
            return Err("content type slugs are invalid");
        }
        if !(1..=24).contains(&self.posts_per_page) {
            return Err("posts per page must be between 1 and 24");
        }
        Ok(())
    }

    pub fn activate_locale(&mut self, requested_locale: &str) -> Result<(), &'static str> {
        let localization = self
            .localization(requested_locale)
            .ok_or("locale has no site localization")?
            .clone();
        self.default_locale = requested_locale.to_owned();
        self.site_name = localization.site_name;
        self.site_description = localization.site_description;
        self.index_page_slug = localization.index_page_slug;
        self.navigation = localization.navigation;
        Ok(())
    }

    pub fn localization(&self, requested_locale: &str) -> Option<&SiteLocalization> {
        self.localizations
            .iter()
            .find(|item| item.locale.eq_ignore_ascii_case(requested_locale))
            .or_else(|| {
                requested_locale.split_once('-').and_then(|(language, _)| {
                    self.localizations
                        .iter()
                        .find(|item| item.locale.eq_ignore_ascii_case(language))
                })
            })
            .or_else(|| {
                self.localizations
                    .iter()
                    .find(|item| item.locale.eq_ignore_ascii_case(&self.site_default_locale))
            })
            .or_else(|| self.localizations.first())
    }

    pub fn locale_codes(&self) -> impl Iterator<Item = &str> {
        self.localizations.iter().map(|item| item.locale.as_str())
    }
}

fn default_locale() -> String {
    "en".into()
}

const fn enabled() -> bool {
    true
}

pub fn valid_locale(locale: &str) -> bool {
    if !(2..=35).contains(&locale.len()) {
        return false;
    }
    let mut parts = locale.split('-');
    let Some(language) = parts.next() else {
        return false;
    };
    (2..=8).contains(&language.len())
        && language.bytes().all(|byte| byte.is_ascii_alphabetic())
        && parts.all(|part| {
            (1..=8).contains(&part.len()) && part.bytes().all(|byte| byte.is_ascii_alphanumeric())
        })
}

pub fn valid_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug.len() <= 160
        && slug
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn valid_href(href: &str) -> bool {
    !href.is_empty()
        && href.len() <= 500
        && !href.chars().any(char::is_whitespace)
        && (href.starts_with('/') || href.starts_with("https://") || href.starts_with("http://"))
}

#[cfg(test)]
mod tests {
    use super::BlogConfig;

    fn settings(localizations: &str) -> BlogConfig {
        serde_json::from_str(&format!(
            r#"{{"default_locale":"en","article_type":"article","page_type":"page","posts_per_page":6,"localizations":{localizations}}}"#
        ))
        .expect("settings deserialize")
    }

    #[test]
    fn validates_localized_settings() {
        let config = settings(
            r#"[{"locale":"en","site_name":"Notes","site_description":"Ideas","index_page_slug":"index","navigation":[{"label":"About","href":"/about"}]},{"locale":"de","site_name":"Notizen","site_description":"Ideen","navigation":[]}]"#,
        );
        assert!(config.validate().is_ok());
    }

    #[test]
    fn requires_a_unique_default_localization() {
        let missing = settings(r#"[{"locale":"de","site_name":"Notizen"}]"#);
        assert!(missing.validate().is_err());
        let duplicate = settings(
            r#"[{"locale":"en","site_name":"Notes"},{"locale":"EN","site_name":"Notes"}]"#,
        );
        assert!(duplicate.validate().is_err());
    }

    #[test]
    fn rejects_unsafe_navigation_urls() {
        let config = settings(
            r#"[{"locale":"en","site_name":"Notes","navigation":[{"label":"Bad","href":"javascript:alert(1)"}]}]"#,
        );
        assert!(config.validate().is_err());
    }

    #[test]
    fn resolves_exact_base_and_default_localizations() {
        let mut config = settings(
            r#"[{"locale":"en","site_name":"Notes"},{"locale":"de","site_name":"Notizen"}]"#,
        );
        config.site_default_locale = "en".into();
        config.activate_locale("de-DE").unwrap();
        assert_eq!(config.site_name, "Notizen");
        config.activate_locale("fr").unwrap();
        assert_eq!(config.site_name, "Notes");
    }
}
