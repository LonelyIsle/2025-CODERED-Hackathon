package handlers

import (
	"encoding/json"
	"errors"
	"net/http"

	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/ai"
)

// ---------- Chat ----------

type chatReq struct {
	Prompt string `json:"prompt"`
}

type chatResp struct {
	OK      bool   `json:"ok"`
	Message string `json:"message"`
}

func ChatHandler(w http.ResponseWriter, r *http.Request) {
	var in chatReq
	if err := json.NewDecoder(r.Body).Decode(&in); err != nil {
		http.Error(w, "bad json", http.StatusBadRequest)
		return
	}
	if in.Prompt == "" {
		http.Error(w, "missing 'prompt'", http.StatusBadRequest)
		return
	}

	msg, err := ai.Chat(in.Prompt) // matches your current ai.Chat signature
	if err != nil {
		http.Error(w, "chat backend error: "+err.Error(), http.StatusBadGateway)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(chatResp{OK: true, Message: msg})
}

// ---------- Embeddings ----------

type embedReq struct {
	Text  string   `json:"text"`  // single
	Texts []string `json:"texts"` // batch
}

type embedRespSingle struct {
	Embedding []float64 `json:"embedding"`
}

type embedRespBatch struct {
	Embeddings [][]float64 `json:"embeddings"`
}

func toFloat64(v []float32) []float64 {
	out := make([]float64, len(v))
	for i, x := range v {
		out[i] = float64(x)
	}
	return out
}

func EmbedHandler(w http.ResponseWriter, r *http.Request) {
	var in embedReq
	if err := json.NewDecoder(r.Body).Decode(&in); err != nil {
		http.Error(w, "bad json", http.StatusBadRequest)
		return
	}

	// Batch: { "texts": ["a","b"] }
	if len(in.Texts) > 0 {
		embs := make([][]float64, 0, len(in.Texts))
		for _, t := range in.Texts {
			vec, err := ai.Embed(t) // single-text embed from your ai package
			if err != nil {
				http.Error(w, "embed backend error: "+err.Error(), http.StatusBadGateway)
				return
			}
			embs = append(embs, toFloat64(vec))
		}
		w.Header().Set("Content-Type", "application/json")
		_ = json.NewEncoder(w).Encode(embedRespBatch{Embeddings: embs})
		return
	}

	// Single: { "text": "..." }
	if in.Text == "" {
		http.Error(w, errors.New("provide 'text' or 'texts'").Error(), http.StatusBadRequest)
		return
	}
	vec, err := ai.Embed(in.Text)
	if err != nil {
		http.Error(w, "embed backend error: "+err.Error(), http.StatusBadGateway)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(embedRespSingle{Embedding: toFloat64(vec)})
}
