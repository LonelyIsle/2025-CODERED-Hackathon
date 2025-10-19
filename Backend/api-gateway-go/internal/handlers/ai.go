package handlers

import (
	"encoding/json"
	"net/http"

	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/ai"
)

// -------- Chat --------

type chatIn struct {
	Prompt string `json:"prompt"`
}

type chatOut struct {
	Output string `json:"output"`
}

func ChatHandler(w http.ResponseWriter, r *http.Request) {
	var in chatIn
	if err := json.NewDecoder(r.Body).Decode(&in); err != nil || in.Prompt == "" {
		http.Error(w, "invalid json: need {\"prompt\":\"...\"}", http.StatusBadRequest)
		return
	}

	reply, err := ai.Chat(in.Prompt)
	if err != nil {
		http.Error(w, "chat failed: "+err.Error(), http.StatusBadGateway)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(chatOut{Output: reply})
}

// -------- Embeddings --------

type embedIn struct {
	Texts []string `json:"texts"`
}

type embedOut struct {
	Embeddings [][]float64 `json:"embeddings"`
}

func EmbedHandler(w http.ResponseWriter, r *http.Request) {
	var in embedIn
	if err := json.NewDecoder(r.Body).Decode(&in); err != nil || len(in.Texts) == 0 {
		http.Error(w, "invalid json: need {\"texts\":[\"...\"]}", http.StatusBadRequest)
		return
	}

	vecs, err := ai.Embed(in.Texts)
	if err != nil {
		http.Error(w, "embed failed: "+err.Error(), http.StatusBadGateway)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(embedOut{Embeddings: vecs})
}