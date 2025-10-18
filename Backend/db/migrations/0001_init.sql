CREATE TABLE companies (
  id SERIAL PRIMARY KEY,
  name TEXT NOT NULL,
  homepage_url TEXT,
  industry TEXT,
  region TEXT,
  created_at TIMESTAMP DEFAULT now()
);

CREATE TABLE documents (
  id BIGSERIAL PRIMARY KEY,
  company_id INT REFERENCES companies(id) ON DELETE CASCADE,
  url TEXT,
  doc_type TEXT,     -- 'report' | 'news' | 'pdf'
  published_at TIMESTAMP,
  text TEXT NOT NULL,
  created_at TIMESTAMP DEFAULT now()
);

-- embeddings split to its own table for performance
CREATE TABLE passages (
  id BIGSERIAL PRIMARY KEY,
  company_id INT REFERENCES companies(id) ON DELETE CASCADE,
  document_id BIGINT REFERENCES documents(id) ON DELETE CASCADE,
  text TEXT NOT NULL,
  published_at TIMESTAMP,
  created_at TIMESTAMP DEFAULT now()
);

-- features aggregated per company (XGBoost inputs)
CREATE TABLE features_company (
  company_id INT PRIMARY KEY REFERENCES companies(id) ON DELETE CASCADE,
  ts_updated TIMESTAMP DEFAULT now(),
  feat_json JSONB NOT NULL
);

-- predictions snapshot for fast fetch
CREATE TABLE predictions (
  company_id INT PRIMARY KEY REFERENCES companies(id) ON DELETE CASCADE,
  impact_score NUMERIC(5,2),
  risk_bucket TEXT,           -- Low | Medium | High
  explanations JSONB,         -- feature importances, SHAP-like
  ts_inferred TIMESTAMP DEFAULT now()
);

CREATE INDEX idx_docs_company ON documents(company_id);
CREATE INDEX idx_passages_company ON passages(company_id);
CREATE INDEX idx_docs_published ON documents(published_at);
CREATE INDEX idx_passages_published ON passages(published_at);