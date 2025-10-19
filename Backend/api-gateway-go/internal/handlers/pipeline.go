package handlers

import (
	"database/sql"
	"encoding/json"
	"net/http"
	"time"

	"github.com/pgvector/pgvector-go"

	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/ai"
	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/db"
	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/util"
)

type backfillReq struct {
	N int `json:"n"`
}
type backfillResp struct {
	Ok           bool `json:"ok"`
	IngestedSeen int  `json:"ingested_seen"`
	DocsCreated  int  `json:"docs_created"`
	ChunksMade   int  `json:"chunks_made"`
	Embedded     int  `json:"embedded"`
	MarkedDone   int  `json:"marked_done"`
}

func Backfill(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
	var req backfillReq
	_ = json.NewDecoder(r.Body).Decode(&req)
	if req.N <= 0 || req.N > 1000 { req.N = 10 }

	type row struct {
		ID         int64
		URL        string
		Body       string
		DocumentID sql.NullInt64
	}
	rows, err := db.Pool.Query(ctx, `
		SELECT id, url, body_text, document_id
		FROM ingested_documents
		WHERE processed = false
		ORDER BY fetched_at DESC
		LIMIT $1`, req.N)
	if err != nil { http.Error(w, err.Error(), 500); return }
	defer rows.Close()

	var items []row
	for rows.Next() {
		var it row
		if err := rows.Scan(&it.ID, &it.URL, &it.Body, &it.DocumentID); err != nil {
			http.Error(w, err.Error(), 500); return
		}
		items = append(items, it)
	}
	if err := rows.Err(); err != nil { http.Error(w, err.Error(), 500); return }

	resp := backfillResp{Ok: true, IngestedSeen: len(items)}
	if len(items) == 0 { writeJSON(w, resp); return }

	tx, err := db.Pool.Begin(ctx)
	if err != nil { http.Error(w, err.Error(), 500); return }
	defer tx.Rollback(ctx)

	type p2e struct {
		ID   int64
		Text string
	}
	var pending []p2e

	for i := range items {
		docID := items[i].DocumentID.Int64
		if !items[i].DocumentID.Valid {
			if err := tx.QueryRow(ctx, `
				INSERT INTO documents (company_id, url, doc_type, published_at, text, created_at)
				VALUES (NULL, $1, 'webpage', NULL, $2, now())
				ON CONFLICT (url) DO UPDATE SET text=EXCLUDED.text
				RETURNING id
			`, items[i].URL, items[i].Body).Scan(&docID); err != nil {
				http.Error(w, "insert document: "+err.Error(), 500); return
			}
			resp.DocsCreated++
			if _, err := tx.Exec(ctx, `UPDATE ingested_documents SET document_id=$1 WHERE id=$2`, docID, items[i].ID); err != nil {
				http.Error(w, "link doc: "+err.Error(), 500); return
			}
		}

		var exists bool
		if err := tx.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM passages WHERE document_id=$1)`, docID).Scan(&exists); err != nil {
			http.Error(w, err.Error(), 500); return
		}
		if !exists {
			chunks := util.ChunkRunes(items[i].Body, 1500)
			now := time.Now()
			for _, c := range chunks {
				var pid int64
				if err := tx.QueryRow(ctx, `
					INSERT INTO passages (company_id, document_id, text, published_at, created_at)
					VALUES (NULL, $1, $2, NULL, $3) RETURNING id
				`, docID, c, now).Scan(&pid); err != nil {
					http.Error(w, "insert passage: "+err.Error(), 500); return
				}
				pending = append(pending, p2e{ID: pid, Text: c})
				resp.ChunksMade++
			}
		}
	}

	const batch = 64
	for i := 0; i < len(pending); i += batch {
		j := i + batch
		if j > len(pending) { j = len(pending) }
		slice := pending[i:j]

		texts := make([]string, len(slice))
		for k := range slice { texts[k] = slice[k].Text }

		vecs, err := ai.EmbedBatch(ctx, texts)
		if err != nil { http.Error(w, "embed: "+err.Error(), 500); return }

		for k := range slice {
			v := pgvector.NewVector(vecs[k])
			if _, err := tx.Exec(ctx, `UPDATE passages SET embedding=$1 WHERE id=$2`, v, slice[k].ID); err != nil {
				http.Error(w, "update embedding: "+err.Error(), 500); return
			}
			resp.Embedded++
		}
	}

	ids := make([]int64, len(items))
	for i := range items { ids[i] = items[i].ID }
	if _, err := tx.Exec(ctx, `UPDATE ingested_documents SET processed=true, updated_at=now() WHERE id = ANY($1)`, ids); err != nil {
		http.Error(w, "mark processed: "+err.Error(), 500); return
	}
	resp.MarkedDone = len(items)

	if err := tx.Commit(ctx); err != nil { http.Error(w, err.Error(), 500); return }
	writeJSON(w, resp)
}

func writeJSON(w http.ResponseWriter, v any) {
	w.Header().Set("content-type", "application/json")
	_ = json.NewEncoder(w).Encode(v)
}
