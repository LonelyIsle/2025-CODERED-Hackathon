-- === Domain data for reports & mitigation suggestions ===

-- Companies you track
CREATE TABLE IF NOT EXISTS companies (
    id              BIGSERIAL PRIMARY KEY,
    name            VARCHAR(255) NOT NULL UNIQUE,
    ticker          VARCHAR(32),
    industry        VARCHAR(128),
    country         VARCHAR(64),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Reports produced by your pipeline (one per company per run)
-- If you installed pgvector in 0002, you can add an embedding column later.
CREATE TABLE IF NOT EXISTS reports (
    id              BIGSERIAL PRIMARY KEY,
    company_id      BIGINT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    period_start    DATE,
    period_end      DATE,
    model_version   VARCHAR(64),
    risk_score      NUMERIC(5,2),                 -- 0..100
    summary_md      TEXT,                         -- markdown summary
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_reports_company_id ON reports(company_id);
CREATE INDEX IF NOT EXISTS idx_reports_created_at ON reports(created_at DESC);

-- Mitigation suggestions your AI recommends (deduped across reports)
CREATE TABLE IF NOT EXISTS mitigation_suggestions (
    id              BIGSERIAL PRIMARY KEY,
    title           VARCHAR(255) NOT NULL,
    description_md  TEXT NOT NULL,                -- markdown description
    category        VARCHAR(64),                  -- e.g., "Energy", "SupplyChain", ...
    effort_score    INT CHECK (effort_score BETWEEN 1 AND 5),
    impact_score    INT CHECK (impact_score BETWEEN 1 AND 5),
    cost_estimate_usd NUMERIC(14,2),              -- optional CAPEX/OPEX estimate
    source          VARCHAR(64),                  -- "LLM", "Rule", "Human"
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_mit_cat ON mitigation_suggestions(category);
CREATE INDEX IF NOT EXISTS idx_mit_scores ON mitigation_suggestions(impact_score, effort_score);

-- Join table: which mitigations were applied/suggested in which report, with priority
CREATE TABLE IF NOT EXISTS report_mitigations (
    report_id       BIGINT NOT NULL REFERENCES reports(id) ON DELETE CASCADE,
    mitigation_id   BIGINT NOT NULL REFERENCES mitigation_suggestions(id) ON DELETE CASCADE,
    priority        INT CHECK (priority BETWEEN 1 AND 100),  -- 1 = highest
    rationale_md    TEXT,
    PRIMARY KEY (report_id, mitigation_id)
);

-- Optional: track status of adoption per company
CREATE TABLE IF NOT EXISTS company_mitigation_status (
    company_id      BIGINT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    mitigation_id   BIGINT NOT NULL REFERENCES mitigation_suggestions(id) ON DELETE CASCADE,
    status          VARCHAR(32) NOT NULL DEFAULT 'planned',  -- planned|in_progress|done|rejected
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    notes_md        TEXT,
    PRIMARY KEY (company_id, mitigation_id)
);