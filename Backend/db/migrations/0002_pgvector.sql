CREATE EXTENSION IF NOT EXISTS vector;

-- choose dimension to match model (e.g., e5-base = 768)
ALTER TABLE passages ADD COLUMN embedding vector(768);

-- common ANN index (HNSW)
CREATE INDEX idx_passages_embedding_hnsw
  ON passages USING hnsw (embedding vector_cosine_ops);

-- quick metadata filters
CREATE INDEX idx_passages_company_published ON passages(company_id, published_at);