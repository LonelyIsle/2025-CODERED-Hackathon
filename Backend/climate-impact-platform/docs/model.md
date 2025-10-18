# Model Separation    
    •	Ingest unstructured data — text from sustainability reports, filings, or even news articles.
	•	Find latent relationships — between company behavior, sector patterns, and long-term environmental outcomes.
	•	Predict dynamic risk — like which companies are likely to have climate-related losses, regulatory exposure, or public backlash before it’s reflected in formal metrics.
	•	Adapt quickly — as new data sources or trends appear, you can fine-tune instead of rewriting rules.

# Predictions
ESG report analysis
Can read thousands of reports and detect greenwashing or tone inconsistency automatically.
Supply chain impact
Learns hidden dependencies between suppliers, geography, and climate risk.
Market sentiment correlation
Links climate-related controversies to financial volatility using textual and numerical data together.
Predictive disclosure modeling
Can estimate future emissions trajectory from past patterns and policy documents.
Cross-company benchmarking
Learns what “good” looks like from peers, instead of you defining thresholds manually.


# Data
companies
id PK | name | homepage_url | industry | region | created_at

documents
id PK | company_id FK | url | doc_type | published_at | text | created_at

passages
id PK | company_id FK | document_id FK | text | published_at | embedding vector(768) | created_at
Indexes: HNSW on embedding, company_id, published_at.

features_company
company_id PK | ts_updated | feat_json JSONB
(aggregates of label probabilities, recency, sector/geography, emissions, etc.)

predictions
company_id PK | impact_score NUMERIC(5,2) | risk_bucket TEXT | explanations JSONB | ts_inferred

mitigations (Knowledge Base)
id PK | sector | scope(1/2/3) | name | summary | prereqs | impact_band | time_to_impact | capex_band | operational_difficulty | sources | embedding vector(768)

recommendations (per company/report)
company_id FK | mitigation_id FK | reason TEXT | confidence NUMERIC(4,3) | evidence_passage_ids BIGINT[] | created_at
PK: (company_id, mitigation_id)