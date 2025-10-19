package handlers

import (
	"encoding/json"
	"net/http"
)

type reportReq struct {
	CompanyID   int64  `json:"company_id"`
	PeriodStart string `json:"period_start,omitempty"`
	PeriodEnd   string `json:"period_end,omitempty"`
}

type reportResp struct {
	OK      bool   `json:"ok"`
	Message string `json:"message"`
}

// GenerateReport — stub implementation so routes compile.
// Replace with your real report pipeline when ready.
func GenerateReport(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(reportResp{
		OK:      true,
		Message: "report generation stub (wire in real logic later)",
	})
}
