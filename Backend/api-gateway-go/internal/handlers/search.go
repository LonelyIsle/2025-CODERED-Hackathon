package handlers

import (
	"encoding/json"
	"net/http"

	"github.com/pgvector/pgvector-go"

	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/ai"
	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/db"
)

type searchReq struct {
	Query     string `json:"query"`
	Limit     int    `json:"limit"`
	CompanyID *int64 `json:"company_id,omitempty"`
}
type searchHit struct {
	ID         int64    `json:"id"`
	DocumentID int64    `json:"document_id"`
	CompanyID  *int64   `json:"company_id,omitempty"`
	Text       string   `json:"text"`
	Score      float32  `json:"score"` // 1 - cosine distance
}
type searchResp struct {
	Ok    bool        `json:"ok"`
	Hits  []searchHit `json:"hits"`
	Count int         `json:"count"`
}

func Search(w http.ResponseWriter, r *http.Request) {
	var req searchReq
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil || req.Query == "" {
		http.Error(w, "invalid body (need query)", 400); return
	}
	if req.Limit <= 0 || req.Limit > 50 { req.Limit = 10 }

	vec, err := ai.EmbedOne(r.Context(), req.Query)
	if err != nil { http.Error(w, "embed query: "+err.Error(), 500); return }
	qv := pgvector.NewVector(vec)

	var rows pgxRows
	if req.CompanyID != nil {
		rows, err = db.Pool.Query(r.Context(), `
			SELECT id, document_id, company_id, text, 1 - (embedding <=> $1) AS score
			FROM passages
			WHERE embedding IS NOT NULL AND company_id = $2
			ORDER BY embedding <=> $1
			LIMIT $3
		`, qv, *req.CompanyID, req.Limit)
	} else {
		rows, err = db.Pool.Query(r.Context(), `
			SELECT id, document_id, company_id, text, 1 - (embedding <=> $1) AS score
			FROM passages
			WHERE embedding IS NOT NULL
			ORDER BY embedding <=> $1
			LIMIT $2
		`, qv, req.Limit)
	}
	if err != nil { http.Error(w, err.Error(), 500); return }
	defer rows.Close()

	var resp searchResp
	resp.Ok = true
	for rows.Next() {
		var h searchHit
		var cid *int64
		if err := rows.Scan(&h.ID, &h.DocumentID, &cid, &h.Text, &h.Score); err != nil {
			http.Error(w, err.Error(), 500); return
		}
		h.CompanyID = cid
		resp.Hits = append(resp.Hits, h)
	}
	resp.Count = len(resp.Hits)
	writeJSON(w, resp)
}

type pgxRows interface {
	Next() bool
	Scan(...any) error
	Close()
	Err() error
}
