package handlers

import (
	"encoding/json"
	"net/http"
)

// AdminDashboard: simple admin ping/status endpoint
func AdminDashboard(w http.ResponseWriter, r *http.Request) {
	type info struct {
		OK      bool   `json:"ok"`
		Message string `json:"message"`
	}
	resp := info{OK: true, Message: "admin alive"}
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(resp)
}