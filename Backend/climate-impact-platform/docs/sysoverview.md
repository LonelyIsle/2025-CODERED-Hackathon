# System Overview

Goal: Generate company-level climate-impact reports with ImpactScore (0–100), RiskBucket, highlighted evidence, and a ranked Mitigation Plan with actionable recommendations, confidence, expected impact band, time-to-impact, and citations.

Key principles
	•	Precompute heavy work (ingest, parse, embed, classify, aggregate) offline.
	•	Serve reports fast via a stateless Go API + Redis cache.
	•	Isolate GPU inference in a C++ (ONNX Runtime + CUDA) microservice sized for an RTX 3060 (12 GB).
	•	Use Postgres + pgvector for one-store structured + vector search.
	•	Add Mitigation Recommender (rule/retrieval over a domain KB; optional LLM for phrasing).

High-level data flow
	1.	Admin/Precompute (Rust worker): scrape→parse→chunk→embed (e5-base)→classify (DeBERTa-base, multi-label)→aggregate features→write to Postgres/pgvector.
	2.	Report (Go API): fetch precomputed features+evidence → optional quick rescoring via C++ service → Mitigation Recommender (rank KB actions) → optional LLM Tailor → return JSON; cache in Redis.

## Architecture

Frontend (Admin + Report)
        │
        ▼
┌───────────────────────────┐
│         Go API            │  (stateless; /admin/*, /report/*)
│  - pgx to Postgres        │
│  - Redis cache            │
│  - calls services via HTTP/gRPC
└────────────┬──────────────┘
             │
   ┌─────────┴───────────┐
   │                     │
┌─────── Rust Worker ────────┐          ┌───── C++ Inference (GPU) ────┐
│ /collect  /process         │  HTTP    │ ONNX Runtime + CUDA (RTX 3060)│
│ - ingest, parse PDFs/HTML  ├─────────►│ - DeBERTa-base classifier     │
│ - embeddings (e5-base)     │          │ - quick rescoring              │
│ - classify (multi-label)   │          └────────────────────────────────┘
│ - aggregate features       │
└───────────┬────────────────┘
            │ writes
            ▼
   PostgreSQL 16 + pgvector
   - companies, documents, passages(+embedding),
     features_company, predictions, mitigations(KB),
     recommendations (per report)

   Redis
   - cached report JSON and top-K search results

Optional: LLM Tailor (llama.cpp server)
- Mistral-7B-Instruct (GGUF, Q4_K_M) for phrasing the mitigation section