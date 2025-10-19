package handlers

import (
	"encoding/json"
	"net/http"
	"os"

	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/util"
)

func GenerateReport(w http.ResponseWriter, r *http.Request) {
	// placeholder: echo body to prove end-to-end
	var payload map[string]any
	_ = json.NewDecoder(r.Body).Decode(&payload)
	resp := map[string]any{
		"ok":       true,
		"payload":  payload,
		"worker":   os.Getenv("WORKER_URL"),
		"inference": os.Getenv("INFERENCE_URL"),
	}
	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(resp)
}

func AdminDashboard(w http.ResponseWriter, r *http.Request) {
	_ = util.JSON(w, http.StatusOK, map[string]any{
		"ok": true,
		"msg": "admin ok",
	})
}