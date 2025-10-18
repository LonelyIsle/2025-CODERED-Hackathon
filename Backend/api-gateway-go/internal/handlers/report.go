package handlers

import (
	"encoding/json"
	"net/http"

	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/clients"
)

type ReportRequest struct {
	Company string `json:"company"`
}

func GenerateReport(w http.ResponseWriter, r *http.Request) {
	var req ReportRequest
	json.NewDecoder(r.Body).Decode(&req)
	resp := clients.RequestInference(req.Company)
	json.NewEncoder(w).Encode(resp)
}