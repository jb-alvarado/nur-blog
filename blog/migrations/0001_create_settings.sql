CREATE TABLE settings (
  id SMALLINT PRIMARY KEY DEFAULT 1 CHECK (id = 1),
  site_name TEXT NOT NULL CHECK (char_length(site_name) BETWEEN 1 AND 120),
  site_description TEXT NOT NULL DEFAULT '' CHECK (char_length(site_description) <= 300),
  favicon_url TEXT,
  article_type VARCHAR(160) NOT NULL DEFAULT 'article' CHECK (article_type ~ '^[a-z0-9-]+$'),
  page_type VARCHAR(160) NOT NULL DEFAULT 'page' CHECK (page_type ~ '^[a-z0-9-]+$'),
  index_page_slug VARCHAR(160) CHECK (index_page_slug ~ '^[a-z0-9-]+$'),
  posts_per_page SMALLINT NOT NULL DEFAULT 6 CHECK (posts_per_page BETWEEN 1 AND 24)
);

INSERT INTO
  settings (
    id,
    site_name,
    site_description,
    favicon_url,
    article_type,
    page_type,
    index_page_slug,
    posts_per_page
  )
VALUES
  (
    1,
    'Nur Blog',
    'Ideas, stories, and useful details.',
    NULL,
    'article',
    'page',
    'index',
    6
  );

CREATE TABLE navigation (
  id BIGSERIAL PRIMARY KEY,
  label TEXT NOT NULL,
  href TEXT NOT NULL,
  position INTEGER NOT NULL
);

INSERT INTO
  navigation (label, href, position)
VALUES
  ('Home', '/', 0),
  ('About', '/about', 1);
