### Public (Report)
	•	GET /report/companies → list companies
	•	GET /report/{company_id} → full report JSON

```json
{
  "company": {"id":1,"name":"Acme"},
  "impact_score": 78.4,
  "risk_bucket": "Medium",
  "highlights": [{"passage":"...","label":"policy","score":0.91}],
  "mitigation_plan": [
    {
      "action":"LED + HVAC retrofits",
      "scope":2,
      "impact_band":"Medium (10–20%)",
      "time_to_impact":"Quick",
      "capex_band":"Low–Med",
      "confidence":0.83,
      "reason":"High transition risk and grid intensity; evidence mentions efficiency gaps.",
      "evidence":[{"passage_id":123,"excerpt":"..."}]
    }
  ],
  "trend":[{"date":"2023-12-31","score":74.8}]
}
```

Admin / Precompute
	•	POST /admin/companies {name, homepage_url, industry, region}
	•	POST /admin/collect {company_id} → Rust worker scrape/parse
	•	POST /admin/process {company_id} → embed/classify/aggregate
	•	GET  /admin/status?company_id= → job status

Internal (Service-to-Service)
	•	Go → C++ Inference: POST /inference/score
Body: { "passages":[...], "task":"climate_multilabel" }
Reply: { "labels":["physical","transition","greenwash","policy"], "probs":[[...],[...]], "latency_ms":137 }
	•	Go → Rust Worker: POST /collect {company_id} / POST /process {company_id}
	•	Go → (optional) LLM Tailor: POST /tailor {actions:[...], evidence:[...], company_ctx:{...}}