# nur-blog

nur-blog is a public, responsive blog frontend for
[nur-cms](https://github.com/jb-alvarado/nur-cms). It is a WebAssembly plugin written in Rust and renders
all public HTML with [Maud](https://maud.lambda.xyz/). Published CMS Pages and
Articles are read through the plugin content API; drafts are never available to
the frontend.

The plugin provides a restrained, blog design with a header, footer
and with a persistent left-side navigation on large screens, an
optional CMS-driven hero, article previews and pagination. Its settings are
maintained in the nur-cms admin interface, so a rebuild is not needed when the
localized site name, site description, navigation, content-type slugs or
favicon changes.

## Features

- Six article previews per overview page by default, with configurable
  pagination.
- CMS Pages at /{page_slug} and Articles at
  /{article_type}/{article_slug}.
- A dynamic `/sitemap.xml` containing the localized home pages and every
  published Page and Article in the blog's available locales. Absolute URLs use
  nur-cms's `server.public_url`, with forwarded request headers as fallback.
- A dynamic `/robots.txt` that references the sitemap and excludes the internal
  nur-cms API, authentication, administration, SSE, and private-file routes.
- An optional Page before the article overview, intended for the home-page
  hero.
- Full-text search across published Articles and Pages in a keyboard-accessible
  modal, with a non-JavaScript search page as fallback.
- Independently configurable website locales with localized site identity,
  index Page and navigation. Fixed English and German interface translations
  are powered by [rust-i18n](https://github.com/longbridge/rust-i18n).
- Category navigation with article counts in the desktop sidebar and as a
  dropdown in the mobile menu. Category filters are retained during pagination
  and language changes.
- Optional media, authors, category, tags and publication date.
- Safe CMS HTML output, including the shared Comrak/Syntect code highlighting
  supplied by nur-cms.
- CSS custom properties for visual themes and a separate override stylesheet.
- A protected admin Web Component for site settings and ordered navigation.

## Requirements

- nur-cms 0.20.x with plugin root routes, admin components, and the
  `published-entry-facets` and `published-entry-references` plugin host calls
  enabled.
- A 5,000,000-instruction override for the blog on pages containing multiple
  rendered article previews:
  `fuel_overrides = { blog = 5_000_000 }` in `[plugins.runtime]`.
- Rust with the wasm32-wasip2 target.
- The accompanying nur-cms Syntect change from this project, which adds
  syntax highlighting to safe Markdown HTML output.

## Configure nur-cms

Enable the plugin and its public and admin capabilities in nur-cms.toml:

```toml
[plugins]
enabled = ["blog"]
additional_directories = ["../nur-blog"]
allow_root_routes = true
allow_admin_components = true

[plugins.runtime]
fuel_overrides = { blog = 5_000_000 }
```

`additional_directories` points to this repository, which acts as the plugin
collection root. nur-cms then discovers the package in `blog/`. Only one
enabled plugin can own the public root route /.

## Build

This repository expects to live next to the nur-cms checkout because the WIT
binding in `blog/src/lib.rs` points to the nur-cms plugin interface.

```sh
rustup target add wasm32-wasip2
cargo build --target wasm32-wasip2 --release
```

The repository root is a Cargo workspace and a nur-cms plugin collection. The
complete plugin crate lives in `blog/`, matching the plugin ID. nur-cms ignores
the Rust and web source files and loads the manifest, generated assets,
migrations, and Wasm component from the same directory. The resulting component
is at:

```text
blog/target/wasm32-wasip2/release/nur_blog.wasm
```

## Install

Deploy the following files while keeping their relative paths:

```text
blog/
├── LICENSE
├── plugin.toml
├── nur_blog.wasm
├── assets/                     # generated assets and theme-overrides.css
└── migrations/
```

Place the directory in a configured nur-cms plugin location. At first load,
the plugin migration creates `settings`, `site_localization`, and `navigation`
in its isolated plugin schema and inserts English and German defaults. Global
settings have typed columns; localized website fields and ordered navigation
entries remain relational rows.

### Create a release archive

The bundle script creates a ready-to-install archive with the manifest,
migrations, generated minified assets, editable theme override, and Wasm
component. For development, `blog/plugin.toml` references the Cargo target
path. In the archive, the component is copied to `blog/nur_blog.wasm` and the
packaged manifest is rewritten to reference that file directly:

```sh
scripts/bundle.sh
```

The resulting archive is written to dist/nur-blog-<version>.tar.gz.

## Admin settings

Open **Blog** in the nur-cms admin menu to configure:

- Default website locale as a BCP 47 code, for example `en`, `de`, or `de-DE`
- Optional multilingual routes and language selection. The selector is shown
  only when this option is enabled and at least two website locales exist.
- Up to 20 website locales, each with its own site name, description,
  optional index-Page slug, and ordered navigation
- Optional favicon URL, for example `/uploads/favicon.svg`; without one, the
  bundled default icon is used. Images can be selected from the existing
  nur-cms media browser.
- Article and Page content-type slugs
- Articles per overview page, from 1 to 24

Navigation destinations may be root-relative paths such as /about, or https://
and http:// URLs. The favicon accepts the same URL forms. Settings and
navigation are validated before they are stored.

## Public routes

| Route                  | Content                               |
| ---------------------- | ------------------------------------- |
| /                      | Hero and newest article previews      |
| /page/2                | Further preview pages                 |
| /search?q=term         | Search published Articles and Pages   |
| /favicon.ico           | Bundled default favicon               |
| /robots.txt            | Crawler rules and sitemap reference   |
| /sitemap.xml           | Dynamic sitemap for published content |
| /{article_type}/{slug} | Full CMS Article                      |
| /{slug}                | Full CMS Page                         |
| /de/                   | Localized home page                   |
| /de/page/2             | Localized preview page                |
| /de/search?q=term      | Localized search                      |
| /de/{article_type}/{slug} | Localized CMS Article             |
| /de/{slug}             | Localized CMS Page                    |

For example, configuring article_type as artikel renders an article at
/artikel/mein-beitrag.

The search opens from the sidebar or mobile header and with `Command+K` on
macOS or `Ctrl+K` on other systems. Results use nur-cms full-text search and
remain limited to published content.

## Internationalization

The configured website locales control localized site names and descriptions,
the hero's index Page, navigation, and the `locale` filter applied to nur-cms
content queries. When multilingual support is enabled, they also control the
public language selector and localized routes. The default locale has no URL
prefix. Each additional locale uses a leading path segment.

Fixed interface text such as search labels, pagination, and accessibility text
is compiled with rust-i18n. English and German are included. Regional variants
such as `de-DE` fall back to their base language and then to English for these
fixed labels. A website locale without a matching compiled translation remains
fully usable and receives the English fixed labels.

Visitors can switch between all configured website locales from the sidebar or
mobile navigation. Internal navigation, articles, pagination, search, and the
sitemap use the same canonical URL form. A language switch keeps the current
overview or search route and its other query parameters. On a Page or Article,
the switch follows nur-cms's content translation group and uses the translated
entry's slug. If that group has no published entry for the selected locale, the
link leads to that locale's home page.

Fixed interface text lives in [blog/locales](blog/locales) and is compiled into
the Wasm component with rust-i18n. The build script generates the admin
component's translation dictionary from the same files, so public and admin
translations share one source. Adding a locale file translates fixed UI text;
adding a website localization in the admin enables its public route and
language-selector entry. Article and Page translations remain managed by
nur-cms.

While multilingual support is enabled, locale codes are reserved as the first
URL segment. A Page slug or article content-type slug such as `de` would then
conflict with the localized German routes and should not be used.

## Themes

[blog/web/blog.css](blog/web/blog.css) contains the complete responsive base
design with a spacious two-column composition and content-focused type hierarchy.
Override its --blog-* custom properties, or component classes, in
[blog/assets/theme-overrides.css](blog/assets/theme-overrides.css). That file is loaded
after the base stylesheet, remains readable, and is never overwritten by the
build. Syntax-highlight colors are exposed through the --blog-code-* custom
properties as well.

The build script minifies CSS with Lightning CSS and JavaScript with Oxc, then
writes deployable files to `blog/assets/*.min.css` and
`blog/assets/*.min.js`. Generated files are ignored by Git. Public stylesheets use the
package version as a query parameter, for example ?v=0.1.0.

## Development checks

```sh
cargo fmt --check
cargo clippy --target wasm32-wasip2 -- -D warnings
cargo test
cargo build --target wasm32-wasip2 --release
scripts/bundle.sh
node --check blog/web/admin.js
```

## License

nur-blog is licensed under the GNU General Public License, version 3. See
[LICENSE](LICENSE) for the complete license text.
