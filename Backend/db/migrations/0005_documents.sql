CREATE TABLE IF NOT EXISTS documents (
  url TEXT PRIMARY KEY,
  fetched_at TIMESTAMPTZ NOT NULL,
  title TEXT,
  description TEXT,
  body_text TEXT NOT NULL,
  content_type TEXT,
  http_status INT NOT NULL
);