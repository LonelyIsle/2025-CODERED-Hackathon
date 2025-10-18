package clients

import (
	"bytes"
	"net/http"
)

func NotifyWorker(company string) {
	body := []byte(`{"company":"` + company + `"}`)
	http.Post("http://127.0.0.1:5002/job", "application/json", bytes.NewReader(body))
}