package clients

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"os"
	"time"
)

type WorkerClient struct {
	base string
	http *http.Client
}

func NewWorkerClient() *WorkerClient {
	base := os.Getenv("WORKER_URL")
	if base == "" {
		base = "http://127.0.0.1:5002"
	}
	return &WorkerClient{
		base: base,
		http: &http.Client{Timeout: 20 * time.Second},
	}
}

type IngestReq struct {
	URL string `json:"url"`
}

type IngestResp struct {
	OK        bool   `json:"ok"`
	URL       string `json:"url"`
	Title     string `json:"title"`
	Bytes     int    `json:"bytes"`
	ElapsedMs int64  `json:"elapsed_ms"`
	Error     string `json:"error,omitempty"`
}

func (c *WorkerClient) IngestURL(ctx context.Context, url string) (*IngestResp, error) {
	payload, _ := json.Marshal(IngestReq{URL: url})
	req, err := http.NewRequestWithContext(ctx, "POST", c.base+"/ingest/url", bytes.NewReader(payload))
	if err != nil { return nil, err }
	req.Header.Set("Content-Type", "application/json")

	resp, err := c.http.Do(req)
	if err != nil { return nil, err }
	defer resp.Body.Close()

	var out IngestResp
	if err := json.NewDecoder(resp.Body).Decode(&out); err != nil {
		return nil, fmt.Errorf("decode: %w", err)
	}
	if resp.StatusCode >= 400 || !out.OK {
		return &out, fmt.Errorf("worker error: %s", out.Error)
	}
	return &out, nil
}