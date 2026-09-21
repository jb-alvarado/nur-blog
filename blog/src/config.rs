use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlogConfig {
    pub site_name: String,
    pub site_description: String,
    #[serde(default)]
    pub favicon_url: Option<String>,
    pub article_type: String,
    pub page_type: String,
    pub index_page_slug: Option<String>,
    pub posts_per_page: usize,
    #[serde(default)]
    pub navigation: Vec<NavigationItem>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NavigationItem {
    pub label: String,
    pub href: String,
}

impl BlogConfig {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.site_name.trim().is_empty() || self.site_name.len() > 120 {
            return Err("site name must contain at most 120 characters");
        }
        if self.site_description.len() > 300 {
            return Err("site description must contain at most 300 characters");
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
        if self
            .index_page_slug
            .as_deref()
            .is_some_and(|slug| !valid_slug(slug))
        {
            return Err("index page slug is invalid");
        }
        if !(1..=24).contains(&self.posts_per_page) {
            return Err("posts per page must be between 1 and 24");
        }
        if self.navigation.len() > 20 {
            return Err("navigation may contain at most 20 links");
        }
        if self.navigation.iter().any(|item| {
            item.label.trim().is_empty() || item.label.len() > 80 || !valid_href(&item.href)
        }) {
            return Err("navigation contains an invalid link");
        }
        Ok(())
    }
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

    #[test]
    fn accepts_settings_without_optional_fields() {
        let config: BlogConfig = serde_json::from_str(
            r#"{"site_name":"Notes","site_description":"","article_type":"article","page_type":"page","index_page_slug":null,"posts_per_page":6}"#,
        )
        .expect("settings deserialize");

        assert!(config.navigation.is_empty());
        assert!(config.favicon_url.is_none());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn accepts_root_relative_favicon_url() {
        let config: BlogConfig = serde_json::from_str(
            r#"{"site_name":"Notes","site_description":"","favicon_url":"/uploads/favicon.svg","article_type":"article","page_type":"page","index_page_slug":null,"posts_per_page":6}"#,
        )
        .expect("settings deserialize");

        assert_eq!(config.favicon_url.as_deref(), Some("/uploads/favicon.svg"));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn rejects_unsafe_navigation_urls() {
        let config: BlogConfig = serde_json::from_str(
            r#"{"site_name":"Notes","site_description":"","article_type":"article","page_type":"page","index_page_slug":null,"posts_per_page":6,"navigation":[{"label":"Bad","href":"javascript:alert(1)"}]}"#,
        )
        .expect("settings deserialize");

        assert!(config.validate().is_err());
    }
}
