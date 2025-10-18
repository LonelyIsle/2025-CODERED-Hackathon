package handlers

import (
	"encoding/json"
	"net/http"
)

func AdminDashboard(w http.ResponseWriter, r *http.Request) {
	json.NewEncoder(w).Encode(map[string]string{
		"status":  "ok",
		"message": "Admin dashboard operational",
	})
}