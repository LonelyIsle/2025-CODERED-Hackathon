package clients

import (
	"bytes"
	"encoding/json"
	"net/http"
)

type InferenceResponse struct {
	Company string  `json:"company"`
	Impact  float64 `json:"impact"`
}

func RequestInference(company string) InferenceResponse {
	body, _ := json.Marshal(map[string]string{"company": company})
	resp, err := http.Post("http://127.0.0.1:5001/infer", "application/json", bytes.NewReader(body))
	if err != nil {
		return InferenceResponse{Company: company, Impact: -1}
	}
	defer resp.Body.Close()
	var out InferenceResponse
	json.NewDecoder(resp.Body).Decode(&out)
	return out
}