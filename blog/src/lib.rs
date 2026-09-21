mod config;
mod content;
mod db;
mod view;

mod bindings {
    wit_bindgen::generate!({
        path: "../../nur-cms/backend/plugins/wit/nur-cms-plugin",
        world: "cms-plugin",
    });
}

use bindings::{
    exports::nur::cms::http_handler::{Guest, PluginError, Request, Response},
    nur::cms::types::Header,
};
use config::BlogConfig;
use percent_encoding::percent_decode_str;

use crate::{
    config::valid_slug,
    content::entry,
    db::handles::configuration,
    view::{article_page, content_page, home_page, response, search_page, search_results},
};

struct Blog;

impl Guest for Blog {
    fn handle(request: Request) -> Result<Response, PluginError> {
        match request.route_id.as_str() {
            "settings" => settings_response(configuration::load_config()?),
            "settings-update" => update_settings(request.body),
            "home" => render_home(0),
            "pagination" => render_pagination(&request),
            "article" => render_article(&request),
            "search" => render_search(&request),
            "page" => render_content_page(&request),
            _ => Err(PluginError::NotFound),
        }
    }
}

fn render_home(offset: usize) -> Result<Response, PluginError> {
    let config = configuration::load_config()?;
    response(
        &config,
        config.site_name.clone(),
        home_page(&config, offset)?,
    )
}

fn render_pagination(request: &Request) -> Result<Response, PluginError> {
    let page = path_param(request, "number").and_then(parse_page_number)?;
    let config = configuration::load_config()?;
    response(
        &config,
        format!("{} · Page {page}", config.site_name),
        home_page(&config, (page - 1) * config.posts_per_page)?,
    )
}

fn render_article(request: &Request) -> Result<Response, PluginError> {
    let config = configuration::load_config()?;
    let article_type = path_param(request, "article_type")?;
    let slug = path_param(request, "slug")?;
    if article_type != config.article_type || !valid_slug(slug) {
        return Err(PluginError::NotFound);
    }

    let article = entry(&config.article_type, slug)?.ok_or(PluginError::NotFound)?;
    let title = content::title(&article, "Article").to_owned();
    response(&config, title, article_page(&article))
}

fn render_search(request: &Request) -> Result<Response, PluginError> {
    let config = configuration::load_config()?;
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
        content::search(&config.article_type, &config.page_type, &query)?
    } else {
        Vec::new()
    };

    if query_parameter(request.query.as_deref(), "fragment")?.as_deref() == Some("1") {
        return Ok(Response {
            status: 200,
            headers: vec![Header {
                name: "content-type".into(),
                value: "text/html; charset=utf-8".into(),
            }],
            body: search_results(&query, &results).into_string().into_bytes(),
        });
    }

    response(&config, "Search".into(), search_page(&query, &results))
}

fn render_content_page(request: &Request) -> Result<Response, PluginError> {
    let config = configuration::load_config()?;
    let slug = path_param(request, "page_slug")?;
    if !valid_slug(slug) {
        return Err(PluginError::NotFound);
    }

    let page = entry(&config.page_type, slug)?.ok_or(PluginError::NotFound)?;
    let title = content::title(&page, "Page").to_owned();
    response(&config, title, content_page(&page))
}

fn path_param<'a>(request: &'a Request, name: &str) -> Result<&'a str, PluginError> {
    request
        .path_params
        .iter()
        .find(|parameter| parameter.name == name)
        .map(|parameter| parameter.value.as_str())
        .ok_or(PluginError::NotFound)
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
    use super::query_parameter;

    #[test]
    fn decodes_search_query_parameter() {
        assert_eq!(
            query_parameter(Some("q=rust+und+wasm%3F&fragment=1"), "q").unwrap(),
            Some("rust und wasm?".into())
        );
    }
}
