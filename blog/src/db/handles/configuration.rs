use crate::{
    bindings::{
        exports::nur::cms::http_handler::PluginError,
        nur::cms::database::{self, NullType, Statement, Value as DatabaseValue},
    },
    config::{BlogConfig, NavigationItem},
};

pub fn load_config() -> Result<BlogConfig, PluginError> {
    let result = database::execute(&Statement {
        sql: "SELECT site_name, site_description, favicon_url, article_type, page_type, \
              index_page_slug, posts_per_page FROM settings WHERE id = 1"
            .into(),
        params: Vec::new(),
    })?;
    let mut config = settings(&result.rows)?;

    let navigation = database::execute(&Statement {
        sql: "SELECT label, href FROM navigation ORDER BY position ASC, id ASC".into(),
        params: Vec::new(),
    })?;
    config.navigation = navigation.rows.iter().filter_map(navigation_item).collect();
    config
        .validate()
        .map_err(|_| PluginError::Failed("blog settings are invalid".into()))?;
    Ok(config)
}

pub fn update(config: BlogConfig) -> Result<(), PluginError> {
    config
        .validate()
        .map_err(|message| PluginError::BadRequest(message.into()))?;

    let mut statements = vec![
        Statement {
            sql: "UPDATE settings SET site_name = $1, site_description = $2, \
                  favicon_url = $3, article_type = $4, page_type = $5, \
                  index_page_slug = $6, posts_per_page = $7 WHERE id = 1"
                .into(),
            params: vec![
                DatabaseValue::Text(config.site_name.clone()),
                DatabaseValue::Text(config.site_description.clone()),
                optional_text(&config.favicon_url),
                DatabaseValue::Text(config.article_type.clone()),
                DatabaseValue::Text(config.page_type.clone()),
                optional_text(&config.index_page_slug),
                DatabaseValue::Integer(config.posts_per_page as i64),
            ],
        },
        Statement {
            sql: "DELETE FROM navigation".into(),
            params: Vec::new(),
        },
    ];
    for (position, item) in config.navigation.iter().enumerate() {
        statements.push(Statement {
            sql: "INSERT INTO navigation (label, href, position) VALUES ($1, $2, $3)".into(),
            params: vec![
                DatabaseValue::Text(item.label.clone()),
                DatabaseValue::Text(item.href.clone()),
                DatabaseValue::Integer(position as i64),
            ],
        });
    }
    database::transaction(&statements)?;

    Ok(())
}

fn settings(rows: &[Vec<DatabaseValue>]) -> Result<BlogConfig, PluginError> {
    let [row] = rows else {
        return Err(settings_unavailable());
    };
    let [
        DatabaseValue::Text(site_name),
        DatabaseValue::Text(site_description),
        favicon_url,
        DatabaseValue::Text(article_type),
        DatabaseValue::Text(page_type),
        index_page_slug,
        DatabaseValue::Integer(posts_per_page),
    ] = row.as_slice()
    else {
        return Err(settings_unavailable());
    };

    Ok(BlogConfig {
        site_name: site_name.clone(),
        site_description: site_description.clone(),
        favicon_url: nullable_text(favicon_url)?,
        article_type: article_type.clone(),
        page_type: page_type.clone(),
        index_page_slug: nullable_text(index_page_slug)?,
        posts_per_page: usize::try_from(*posts_per_page).map_err(|_| settings_unavailable())?,
        navigation: Vec::new(),
    })
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

fn navigation_item(row: &Vec<DatabaseValue>) -> Option<NavigationItem> {
    match row.as_slice() {
        [DatabaseValue::Text(label), DatabaseValue::Text(href)] => Some(NavigationItem {
            label: label.to_owned(),
            href: href.to_owned(),
        }),
        _ => None,
    }
}
