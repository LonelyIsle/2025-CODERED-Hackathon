package ai

import (
	"bytes"
	"encoding/json"
	"errors"
	"net"
	"net/http"
	"os"
	"time"
)

var httpClient = &http.Client{
	Timeout: 60 * time.Second,
	Transport: &http.Transport{
		DialContext: (&net.Dialer{
			Timeout:   10 * time.Second,
			KeepAlive: 60 * time.Second,
		}).DialContext,
		MaxIdleConns:        100,
		IdleConnTimeout:     90 * time.Second,
		TLSHandshakeTimeout: 10 * time.Second,
	},
}

func ollamaURL() string {
	if v := os.Getenv("OLLAMA_BASE_URL"); v != "" {
		return v
	}
	// default to local podman/docker
	return "http://127.0.0.1:11434"
}

func ollamaModel() string {
	if v := os.Getenv("OLLAMA_MODEL"); v != "" {
		return v
	}
	return "llama3:8b"
}

type ollamaGenerateReq struct {
	Model  string `json:"model"`
	Prompt string `json:"prompt"`
	Stream bool   `json:"stream"`
	// You can add temperature/top_p/etc if you want:
	// Options map[string]any `json:"options,omitempty"`
}

type ollamaGenerateResp struct {
	Response string `json:"response"`
	Done     bool   `json:"done"`
	// other fields omitted
}

func Chat(prompt string) (string, error) {
	payload := ollamaGenerateReq{
		Model:  ollamaModel(),
		Prompt: prompt,
		Stream: false, // easier for API; switch to true if you want SSE/stream
	}

	b, _ := json.Marshal(payload)
	req, err := http.NewRequest("POST", ollamaURL()+"/api/generate", bytes.NewReader(b))
	if err != nil {
		return "", err
	}
	req.Header.Set("Content-Type", "application/json")

	res, err := httpClient.Do(req)
	if err != nil {
		return "", err
	}
	defer res.Body.Close()

	if res.StatusCode < 200 || res.StatusCode >= 300 {
		return "", errors.New("ollama generate failed with status " + res.Status)
	}

	var og ollamaGenerateResp
	if err := json.NewDecoder(res.Body).Decode(&og); err != nil {
		return "", err
	}

	return og.Response, nil
}