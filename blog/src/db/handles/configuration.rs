use crate::{
    bindings::{
        exports::nur::cms::http_handler::PluginError,
        nur::cms::database::{self, NullType, Statement, Value as DatabaseValue},
    },
    config::{BlogConfig, NavigationItem, SiteLocalization},
};
use std::fmt::Write;

pub fn load_config() -> Result<BlogConfig, PluginError> {
    let result = database::execute(&Statement {
        sql: "SELECT default_locale, multilingual_enabled, favicon_url, article_type, page_type, \
              posts_per_page, locale, site_name, site_description, index_page_slug, label, href \
              FROM settings CROSS JOIN site_localization LEFT JOIN navigation USING (locale) \
              ORDER BY locale ASC, position ASC"
            .into(),
        params: Vec::new(),
    })?;
    let mut config = settings(&result.rows)?;
    for row in &result.rows {
        append_localization(&mut config.localizations, row)?;
    }

    config.site_default_locale = config.default_locale.clone();
    config
        .validate()
        .map_err(|_| PluginError::Failed("blog settings are invalid".into()))?;
    let default_locale = config.default_locale.clone();
    config
        .activate_locale(&default_locale)
        .map_err(|_| PluginError::Failed("blog localization is unavailable".into()))?;
    Ok(config)
}

pub fn update(mut config: BlogConfig) -> Result<(), PluginError> {
    config
        .validate()
        .map_err(|message| PluginError::BadRequest(message.into()))?;
    config.default_locale = config
        .localizations
        .iter()
        .find(|item| item.locale.eq_ignore_ascii_case(&config.default_locale))
        .map(|item| item.locale.clone())
        .ok_or_else(|| PluginError::BadRequest("default locale requires a localization".into()))?;

    let mut statements = vec![
        Statement {
            sql: "UPDATE settings SET default_locale = $1, multilingual_enabled = $2, favicon_url = $3, article_type = $4, \
                  page_type = $5, posts_per_page = $6 WHERE id = 1"
                .into(),
            params: vec![
                DatabaseValue::Text(config.default_locale.clone()),
                DatabaseValue::Boolean(config.multilingual_enabled),
                optional_text(&config.favicon_url),
                DatabaseValue::Text(config.article_type.clone()),
                DatabaseValue::Text(config.page_type.clone()),
                DatabaseValue::Integer(config.posts_per_page as i64),
            ],
        },
        Statement {
            sql: "DELETE FROM site_localization".into(),
            params: Vec::new(),
        },
    ];
    statements.push(localization_insert(&config.localizations));
    let navigation = config
        .localizations
        .iter()
        .flat_map(|localization| {
            localization
                .navigation
                .iter()
                .enumerate()
                .map(|(position, item)| (localization.locale.as_str(), position, item))
        })
        .collect::<Vec<_>>();
    statements.extend(navigation.chunks(32).map(navigation_insert));
    database::transaction(&statements)?;
    Ok(())
}

fn localization_insert(localizations: &[SiteLocalization]) -> Statement {
    let mut sql = String::from(
        "INSERT INTO site_localization (locale, site_name, site_description, index_page_slug) VALUES ",
    );
    let mut params = Vec::with_capacity(localizations.len() * 4);
    append_value_groups(&mut sql, localizations.len(), 4);
    for localization in localizations {
        params.extend([
            DatabaseValue::Text(localization.locale.clone()),
            DatabaseValue::Text(localization.site_name.clone()),
            DatabaseValue::Text(localization.site_description.clone()),
            optional_text(&localization.index_page_slug),
        ]);
    }
    Statement { sql, params }
}

fn navigation_insert(rows: &[(&str, usize, &NavigationItem)]) -> Statement {
    let mut sql = String::from("INSERT INTO navigation (locale, label, href, position) VALUES ");
    let mut params = Vec::with_capacity(rows.len() * 4);
    append_value_groups(&mut sql, rows.len(), 4);
    for (locale, position, item) in rows {
        params.extend([
            DatabaseValue::Text((*locale).to_owned()),
            DatabaseValue::Text(item.label.clone()),
            DatabaseValue::Text(item.href.clone()),
            DatabaseValue::Integer(*position as i64),
        ]);
    }
    Statement { sql, params }
}

fn append_value_groups(sql: &mut String, rows: usize, columns: usize) {
    for row in 0..rows {
        if row > 0 {
            sql.push_str(", ");
        }
        sql.push('(');
        for column in 0..columns {
            if column > 0 {
                sql.push_str(", ");
            }
            write!(sql, "${}", row * columns + column + 1)
                .expect("writing to a String cannot fail");
        }
        sql.push(')');
    }
}

fn settings(rows: &[Vec<DatabaseValue>]) -> Result<BlogConfig, PluginError> {
    let Some(row) = rows.first() else {
        return Err(settings_unavailable());
    };
    let [
        DatabaseValue::Text(default_locale),
        DatabaseValue::Boolean(multilingual_enabled),
        favicon_url,
        DatabaseValue::Text(article_type),
        DatabaseValue::Text(page_type),
        DatabaseValue::Integer(posts_per_page),
        ..,
    ] = row.as_slice()
    else {
        return Err(settings_unavailable());
    };

    Ok(BlogConfig {
        default_locale: default_locale.clone(),
        multilingual_enabled: *multilingual_enabled,
        favicon_url: nullable_text(favicon_url)?,
        article_type: article_type.clone(),
        page_type: page_type.clone(),
        posts_per_page: usize::try_from(*posts_per_page).map_err(|_| settings_unavailable())?,
        localizations: Vec::new(),
        site_name: String::new(),
        site_description: String::new(),
        index_page_slug: None,
        navigation: Vec::new(),
        active_category: None,
        current_url: "/".into(),
        site_default_locale: default_locale.clone(),
        language_paths: Vec::new(),
    })
}

fn append_localization(
    localizations: &mut Vec<SiteLocalization>,
    row: &[DatabaseValue],
) -> Result<(), PluginError> {
    let [
        _,
        _,
        _,
        _,
        _,
        _,
        DatabaseValue::Text(locale),
        DatabaseValue::Text(site_name),
        DatabaseValue::Text(site_description),
        index_page_slug,
        label,
        href,
    ] = row
    else {
        return Err(settings_unavailable());
    };
    if localizations.last().map(|item| item.locale.as_str()) != Some(locale) {
        localizations.push(SiteLocalization {
            locale: locale.clone(),
            site_name: site_name.clone(),
            site_description: site_description.clone(),
            index_page_slug: nullable_text(index_page_slug)?,
            navigation: Vec::new(),
        });
    }
    match (label, href) {
        (DatabaseValue::Text(label), DatabaseValue::Text(href)) => {
            localizations
                .last_mut()
                .ok_or_else(settings_unavailable)?
                .navigation
                .push(NavigationItem {
                    label: label.clone(),
                    href: href.clone(),
                });
            Ok(())
        }
        (DatabaseValue::Null(NullType::Text), DatabaseValue::Null(NullType::Text)) => Ok(()),
        _ => Err(settings_unavailable()),
    }
}

fn nullable_text(value: &DatabaseValue) -> Result<Option<String>, PluginError> {
    match value {
        DatabaseValue::Text(value) => Ok(Some(value.clone())),
        DatabaseValue::Null(NullType::Text) => Ok(None),
        _ => Err(settings_unavailable()),
    }
}

fn optional_text(value: &Option<String>) -> DatabaseValue {
    value
        .as_ref()
        .map_or(DatabaseValue::Null(NullType::Text), |value| {
            DatabaseValue::Text(value.clone())
        })
}

fn settings_unavailable() -> PluginError {
    PluginError::Failed("blog settings are unavailable".into())
}

#[cfg(test)]
mod tests {
    use super::navigation_insert;
    use crate::config::NavigationItem;

    #[test]
    fn bulk_navigation_insert_has_one_parameter_per_column() {
        let first = NavigationItem {
            label: "Home".into(),
            href: "/".into(),
        };
        let second = NavigationItem {
            label: "About".into(),
            href: "/about".into(),
        };
        let statement = navigation_insert(&[("en", 0, &first), ("en", 1, &second)]);

        assert_eq!(statement.params.len(), 8);
        assert!(statement.sql.ends_with("($5, $6, $7, $8)"));
    }
}
