package ai

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"os"
	"time"
)

// Public helpers used by handlers.
func EmbedText(s string) ([]float32, error) { return embedOnce(s) }
func Embed(s string) ([]float32, error)     { return embedOnce(s) }

// ---- Internal ----

type teiRequest struct {
	Input any `json:"input"` // allow string or []string
}

type teiRespEmbeddings struct {
	Embeddings [][]float32 `json:"embeddings"`
}

type openAIStyleResp struct {
	Data []struct {
		Embedding []float32 `json:"embedding"`
	} `json:"data"`
}

func embedOnce(text string) ([]float32, error) {
	base := os.Getenv("EMBEDDINGS_URL")
	if base == "" {
		return nil, errors.New("EMBEDDINGS_URL not set")
	}

	// Try common TEI path first, then OpenAI-style as a fallback.
	paths := []string{"/embed", "/v1/embeddings"}
	var lastErr error

	for _, p := range paths {
		u := trimSlash(base) + p
		vec, err := callEmbeddings(u, text)
		if err == nil && len(vec) > 0 {
			return vec, nil
		}
		if err != nil {
			lastErr = err
		}
	}

	if lastErr != nil {
		return nil, lastErr
	}
	return nil, fmt.Errorf("no embeddings returned from %s", base)
}

func callEmbeddings(url, text string) ([]float32, error) {
	body, _ := json.Marshal(teiRequest{Input: text})

	req, err := http.NewRequest(http.MethodPost, url, bytes.NewReader(body))
	if err != nil { return nil, err }
	req.Header.Set("Content-Type", "application/json")

	client := &http.Client{ Timeout: 30 * time.Second }
	res, err := client.Do(req)
	if err != nil { return nil, err }
	defer res.Body.Close()

	if res.StatusCode < 200 || res.StatusCode >= 300 {
		b, _ := io.ReadAll(res.Body)
		return nil, fmt.Errorf("embeddings http %d: %s", res.StatusCode, string(b))
	}

	// Try TEI response shape: {"embeddings":[[...]]}
	var tei teiRespEmbeddings
	if err := json.NewDecoder(res.Body).Decode(&tei); err == nil && len(tei.Embeddings) > 0 {
		return tei.Embeddings[0], nil
	}

	// If first decode consumed the body, we need to re-read it; safer to read once:
	// Re-issue request to parse OpenAI-style (rarely needed but robust)
	req2, _ := http.NewRequest(http.MethodPost, url, bytes.NewReader(body))
	req2.Header.Set("Content-Type", "application/json")
	res2, err := client.Do(req2)
	if err != nil { return nil, err }
	defer res2.Body.Close()

	var oa openAIStyleResp
	if err := json.NewDecoder(res2.Body).Decode(&oa); err == nil && len(oa.Data) > 0 {
		return oa.Data[0].Embedding, nil
	}

	return nil, errors.New("unable to parse embeddings response")
}

func trimSlash(s string) string {
	if s == "" { return s }
	if s[len(s)-1] == '/' { return s[:len(s)-1] }
	return s
}
