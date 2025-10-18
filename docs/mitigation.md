Mitigation Recommender (Design)

Inputs: risk profile (label probs), sector/region, electricity grid CI, emissions mix if available, top-K evidence passages, KB embeddings.
Ranking signals (fast & deterministic):
	•	alignment = dot(risk_vector, action_vector)
	•	feasibility from prereqs vs extracted signals
	•	context boosts (sector/region/grid)
	•	evidence strength (count, confidence, freshness)

Outputs: top N actions with confidence and evidence links.
Optional Tailor: pass the structured plan + 2–3 evidence snippets/action to a small GGUF LLM for executive-style wording (temperature 0.2–0.4, max tokens ~512–768).
