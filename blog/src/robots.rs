use crate::{
    bindings::{
        exports::nur::cms::http_handler::{PluginError, Request, Response},
        nur::cms::{configuration, types::Header},
    },
    sitemap::preferred_origin,
};

const DISALLOWED_PATHS: &[&str] = &["/admin", "/api", "/auth", "/files", "/sse"];

pub fn response(request: &Request) -> Result<Response, PluginError> {
    let origin = preferred_origin(configuration::public_url(), &request.headers)?;

    Ok(Response {
        status: 200,
        headers: vec![
            Header {
                name: "content-type".into(),
                value: "text/plain; charset=utf-8".into(),
            },
            Header {
                name: "cache-control".into(),
                value: "public, max-age=3600".into(),
            },
        ],
        body: render(&origin).into_bytes(),
    })
}

fn render(origin: &str) -> String {
    let mut body = String::from("User-agent: *\nAllow: /\n");
    for path in DISALLOWED_PATHS {
        body.push_str("Disallow: ");
        body.push_str(path);
        body.push('\n');
    }
    body.push_str("Sitemap: ");
    body.push_str(origin);
    body.push_str("/sitemap.xml\n");
    body
}

#[cfg(test)]
mod tests {
    use super::render;

    #[test]
    fn excludes_internal_routes_and_keeps_public_assets_available() {
        let robots = render("https://blog.example.org");

        for path in ["/admin", "/api", "/auth", "/files", "/sse"] {
            assert!(robots.contains(&format!("Disallow: {path}\n")));
        }
        assert!(!robots.contains("Disallow: /uploads"));
        assert!(!robots.contains("Disallow: /p"));
        assert!(robots.contains("Sitemap: https://blog.example.org/sitemap.xml\n"));
    }
}
