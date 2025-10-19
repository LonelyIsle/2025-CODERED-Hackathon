package ai

import (
	"bytes"
	"encoding/json"
	"errors"
	"net/http"
	"os"
)

func embeddingsURL() string {
	// Prefer explicit EMBEDDINGS_BASE_URL; otherwise fallback to INFERENCE_URL if you set it,
	// else default to the common port you’re using.
	if v := os.Getenv("EMBEDDINGS_BASE_URL"); v != "" {
		return v
	}
	if v := os.Getenv("INFERENCE_URL"); v != "" {
		return v
	}
	return "http://127.0.0.1:8000"
}

type embedReq struct {
	Inputs []string `json:"inputs"`
}

func Embed(texts []string) ([][]float64, error) {
	if len(texts) == 0 {
		return [][]float64{}, nil
	}

	body, _ := json.Marshal(embedReq{Inputs: texts})

	req, err := http.NewRequest("POST", embeddingsURL()+"/embed", bytes.NewReader(body))
	if err != nil {
		return nil, err
	}
	req.Header.Set("Content-Type", "application/json")

	res, err := httpClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer res.Body.Close()

	if res.StatusCode < 200 || res.StatusCode >= 300 {
		return nil, errors.New("embeddings request failed with status " + res.Status)
	}

	var vectors [][]float64
	if err := json.NewDecoder(res.Body).Decode(&vectors); err != nil {
		return nil, err
	}
	return vectors, nil
}