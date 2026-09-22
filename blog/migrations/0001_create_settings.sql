CREATE TABLE settings (
  id SMALLINT PRIMARY KEY DEFAULT 1 CHECK (id = 1),
  default_locale VARCHAR(35) NOT NULL DEFAULT 'en' CHECK (default_locale ~ '^[A-Za-z]{2,8}(-[A-Za-z0-9]{1,8})*$'),
  multilingual_enabled BOOLEAN NOT NULL DEFAULT TRUE,
  favicon_url TEXT,
  article_type VARCHAR(160) NOT NULL DEFAULT 'article' CHECK (article_type ~ '^[a-z0-9-]+$'),
  page_type VARCHAR(160) NOT NULL DEFAULT 'page' CHECK (page_type ~ '^[a-z0-9-]+$'),
  posts_per_page SMALLINT NOT NULL DEFAULT 6 CHECK (posts_per_page BETWEEN 1 AND 24)
);

INSERT INTO settings (id, default_locale, multilingual_enabled, favicon_url, article_type, page_type, posts_per_page)
VALUES (1, 'en', TRUE, NULL, 'article', 'page', 6);

CREATE TABLE site_localization (
  locale VARCHAR(35) PRIMARY KEY CHECK (locale ~ '^[A-Za-z]{2,8}(-[A-Za-z0-9]{1,8})*$'),
  site_name TEXT NOT NULL CHECK (char_length(site_name) BETWEEN 1 AND 120),
  site_description TEXT NOT NULL DEFAULT '' CHECK (char_length(site_description) <= 300),
  index_page_slug VARCHAR(160) CHECK (index_page_slug ~ '^[a-z0-9-]+$')
);

INSERT INTO site_localization (locale, site_name, site_description, index_page_slug)
VALUES
  ('en', 'Nur Blog', 'Ideas, stories, and useful details.', 'index'),
  ('de', 'Nur Blog', 'Ideen, Geschichten und nützliche Details.', 'index');

CREATE TABLE navigation (
  id BIGSERIAL PRIMARY KEY,
  locale VARCHAR(35) NOT NULL REFERENCES site_localization(locale) ON DELETE CASCADE,
  label TEXT NOT NULL CHECK (char_length(label) BETWEEN 1 AND 80),
  href TEXT NOT NULL CHECK (char_length(href) BETWEEN 1 AND 500),
  position INTEGER NOT NULL,
  UNIQUE (locale, position)
);

INSERT INTO navigation (locale, label, href, position)
VALUES
  ('en', 'Home', '/', 0),
  ('en', 'About', '/about', 1),
  ('de', 'Startseite', '/', 0),
  ('de', 'Über uns', '/about', 1);
