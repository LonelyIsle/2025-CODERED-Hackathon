package ai

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"os"
	"time"
)

type teiReq struct {
	Inputs []string `json:"inputs"`
}
type teiResp struct {
	Embeddings [][]float32 `json:"embeddings"`
}

var httpc = &http.Client{ Timeout: 30 * time.Second }

func EmbedBatch(ctx context.Context, texts []string) ([][]float32, error) {
	if len(texts) == 0 { return nil, nil }
	url := os.Getenv("EMBEDDINGS_URL")
	if url == "" { url = "http://127.0.0.1:8000" }

	b, _ := json.Marshal(teiReq{Inputs: texts})
	req, _ := http.NewRequestWithContext(ctx, http.MethodPost, url, bytes.NewReader(b))
	req.Header.Set("content-type", "application/json")

	res, err := httpc.Do(req)
	if err != nil { return nil, err }
	defer res.Body.Close()
	if res.StatusCode/100 != 2 {
		return nil, fmt.Errorf("embeddings http %d", res.StatusCode)
	}
	var out teiResp
	if err := json.NewDecoder(res.Body).Decode(&out); err != nil {
		return nil, err
	}
	if len(out.Embeddings) != len(texts) {
		return nil, fmt.Errorf("embedding size mismatch: got %d want %d", len(out.Embeddings), len(texts))
	}
	return out.Embeddings, nil
}

func EmbedOne(ctx context.Context, text string) ([]float32, error) {
	vecs, err := EmbedBatch(ctx, []string{text})
	if err != nil { return nil, err }
	if len(vecs) == 0 { return nil, fmt.Errorf("no embedding returned") }
	return vecs[0], nil
}
