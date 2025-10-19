package handlers

import (
	"encoding/json"
	"net/http"

	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/clients"
)

type ingestReq struct {
	URL string `json:"url"`
}

func IngestURL(w http.ResponseWriter, r *http.Request) {
	var in ingestReq
	if err := json.NewDecoder(r.Body).Decode(&in); err != nil || in.URL == "" {
		http.Error(w, "bad request", http.StatusBadRequest)
		return
	}
	cli := clients.NewWorkerClient()
	out, err := cli.IngestURL(r.Context(), in.URL)
	if err != nil {
		w.WriteHeader(http.StatusBadGateway)
		_ = json.NewEncoder(w).Encode(map[string]any{"ok": false, "error": err.Error()})
		return
	}
	_ = json.NewEncoder(w).Encode(out)
}