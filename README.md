# nur-blog

nur-blog is a public, responsive blog frontend for
[nur-cms](https://github.com/jb-alvarado/nur-cms). It is a WebAssembly plugin written in Rust and renders
all public HTML with [Maud](https://maud.lambda.xyz/). Published CMS Pages and
Articles are read through the plugin content API; drafts are never available to
the frontend.

The plugin provides a restrained, neutral blog design with a header, footer,
left-side navigation, an optional CMS-driven hero, article previews and
pagination. Its settings are maintained in the nur-cms admin interface, so a
rebuild is not needed when the site name, content-type slugs or navigation
changes, including the favicon.

## Features

- Six article previews per overview page by default, with configurable
  pagination.
- CMS Pages at /{page_slug} and Articles at
  /{article_type}/{article_slug}.
- An optional Page before the article overview, intended for the home-page
  hero.
- Full-text search across published Articles and Pages in a keyboard-accessible
  modal, with a non-JavaScript search page as fallback.
- Optional media, authors, category, tags and publication date.
- Safe CMS HTML output, including the shared Comrak/Syntect code highlighting
  supplied by nur-cms.
- CSS custom properties for visual themes and a separate override stylesheet.
- A protected admin Web Component for site settings and ordered navigation.

## Requirements

- nur-cms 0.19.x or 0.20.x with plugin root routes and admin components enabled.
- Rust with the wasm32-wasip2 target.
- The accompanying nur-cms Syntect change from this project, which adds
  syntax highlighting to safe Markdown HTML output.

## Configure nur-cms

Enable the plugin and its public and admin capabilities in nur-cms.toml:

~~~toml
[plugins]
enabled = ["blog"]
additional_directories = ["../nur-blog"]
allow_root_routes = true
allow_admin_components = true
~~~

`additional_directories` points to this repository, which acts as the plugin
collection root. nur-cms then discovers the package in `blog/`. Only one
enabled plugin can own the public root route /.

## Build

This repository expects to live next to the nur-cms checkout because the WIT
binding in `blog/src/lib.rs` points to the nur-cms plugin interface.

~~~sh
rustup target add wasm32-wasip2
cargo build --target wasm32-wasip2 --release
~~~

The repository root is a Cargo workspace and a nur-cms plugin collection. The
complete plugin crate lives in `blog/`, matching the plugin ID. nur-cms ignores
the Rust and web source files and loads the manifest, generated assets,
migrations, and Wasm component from the same directory. The resulting component
is at:

~~~text
blog/target/wasm32-wasip2/release/nur_blog.wasm
~~~

## Install

Deploy the following files while keeping their relative paths:

~~~text
blog/
├── LICENSE
├── plugin.toml
├── nur_blog.wasm
├── assets/                     # generated assets and theme-overrides.css
└── migrations/
~~~

Place the directory in a configured nur-cms plugin location. At first load,
the plugin migration creates `settings` and `navigation` in its isolated plugin
schema and inserts sensible defaults. Each scalar setting has its own typed
column; ordered navigation entries remain relational rows.

### Create a release archive

The bundle script creates a ready-to-install archive with the manifest,
migrations, generated minified assets, editable theme override, and Wasm
component. For development, `blog/plugin.toml` references the Cargo target
path. In the archive, the component is copied to `blog/nur_blog.wasm` and the
packaged manifest is rewritten to reference that file directly:

~~~sh
scripts/bundle.sh
~~~

The resulting archive is written to dist/nur-blog-<version>.tar.gz.

## Admin settings

Open **Blog** in the nur-cms admin menu to configure:

- Site name and description
- Optional favicon URL, for example `/uploads/favicon.svg`; without one, the
  bundled default icon is used. Images can be selected from the existing
  nur-cms media browser.
- Article and Page content-type slugs
- Optional index-Page slug for the hero
- Articles per overview page, from 1 to 24
- Ordered navigation links

Navigation destinations may be root-relative paths such as /about, or https://
and http:// URLs. The favicon accepts the same URL forms. Settings and
navigation are validated before they are stored.

## Public routes

| Route | Content |
| --- | --- |
| / | Hero and newest article previews |
| /page/2 | Further preview pages |
| /search?q=term | Search published Articles and Pages |
| /favicon.ico | Bundled default favicon |
| /{article_type}/{slug} | Full CMS Article |
| /{slug} | Full CMS Page |

For example, configuring article_type as artikel renders an article at
/artikel/mein-beitrag.

The search opens from the sidebar or mobile header and with `Command+K` on
macOS or `Ctrl+K` on other systems. Results use nur-cms full-text search and
remain limited to published content.

## Themes

[blog/web/blog.css](blog/web/blog.css) contains the complete responsive base design.
Override its --blog-* custom properties, or component classes, in
[blog/assets/theme-overrides.css](blog/assets/theme-overrides.css). That file is loaded
after the base stylesheet, remains readable, and is never overwritten by the
build. Syntax-highlight colors are exposed through the --blog-code-* custom
properties as well.

The build script minifies CSS with Lightning CSS and JavaScript with Oxc, then
writes deployable files to blog/assets/*.min.css and
blog/assets/*.min.js. Generated files are ignored by Git. Public stylesheets use the
package version as a query parameter, for example ?v=0.1.0.

## Development checks

~~~sh
cargo fmt --check
cargo clippy --target wasm32-wasip2 -- -D warnings
cargo test
cargo build --target wasm32-wasip2 --release
scripts/bundle.sh
node --check blog/web/admin.js
~~~

## License

nur-blog is licensed under the GNU General Public License, version 3. See
[LICENSE](LICENSE) for the complete license text.
