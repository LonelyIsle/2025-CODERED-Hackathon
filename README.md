# Hacking The Climate

## Overview
**Hacking The Climate** is a platform that generates company-level climate-impact reports. Each report provides:  

- **ImpactScore (0–100)**  
- **RiskBucket**  
- **Highlighted evidence**  
- **Ranked Mitigation Plan** with actionable recommendations, confidence, expected impact, time-to-impact, and citations  

The system is designed to process large amounts of environmental and financial data efficiently and provide actionable insights.

---

## Key Features

- **Precomputed Analysis** – Ingest, parse, embed, classify, and aggregate offline for faster reporting.  
- **Fast Reports** – Serve JSON reports via a stateless Go API with Redis caching.  
- **GPU Inference** – Isolated C++ service for quick rescoring using ONNX Runtime + CUDA (optimized for RTX 3060).  
- **Structured + Vector Storage** – PostgreSQL + pgvector for combined structured and embedding-based search.  
- **Mitigation Recommender** – Suggests actionable steps based on domain knowledge and optional LLM phrasing.

---

## Architecture

- **Frontend** – React/Next.js dashboard and admin panel.  
- **API Gateway** – Go server serving report data and admin functions.  
- **Data Worker** – Rust worker that scrapes, parses, embeds, classifies, and aggregates features.  
- **Inference Service** – C++ microservice for quick scoring using GPU acceleration.  
- **Database** – PostgreSQL with pgvector, storing companies, documents, passages, features, predictions, and mitigation KB.  
- **Cache** – Redis for report JSON and top-K search results.  
- **Optional** – LLM Tailor (Mistral 7B) for natural language phrasing of mitigation recommendations.

---

## Predictions & Insights

- ESG report analysis  
- Supply chain climate impact detection  
- Market sentiment correlation  
- Predictive disclosure modeling  
- Cross-company benchmarking  

---

## Getting Started

1. Clone the repository  
2. Set up environment variables in `.env`  
3. Start the backend:  
```bash
cd Backend/api-gateway-go
go run main.go
```
4. View the frontend:
```bash
cd frontend
npm run dev
```
