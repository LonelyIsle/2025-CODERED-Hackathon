package main

import (
	"log"
	"net/http"
	"os"
	"time"

	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"

	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/auth"
	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/cache"
	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/db"
	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/handlers"
	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/security"
)

func main() {
	_ = loadDotEnv("../.env") // load Backend/.env if present

	if err := db.Init(); err != nil {
		log.Fatalf("db init: %v", err)
	}
	cache.Init(os.Getenv("VALKEY_ADDR"), mustAtoiEnv("VALKEY_DB", 0))

	r := chi.NewRouter()
	r.Use(middleware.Logger)
	r.Use(security.CSRFMiddleware)                      // double-submit cookie unless DISABLE_CSRF=true
	r.Use(security.RateLimitMiddleware(120, time.Minute)) // 120 req/min per IP

	// health
	r.Get("/api/ping", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte(`{"ok":true,"pong":true}`))
	})

	// CSRF token endpoint (double submit cookie)
r.Get("/api/auth/csrf", security.IssueCSRFToken)

	// auth
	r.Post("/api/auth/login", auth.Login)
	r.Post("/api/auth/logout", auth.Logout)
	r.Get("/api/auth/me", func(w http.ResponseWriter, req *http.Request) {
		auth.RequireAuth(http.HandlerFunc(auth.Me)).ServeHTTP(w, req)
	})

	// reports/admin
	r.Post("/api/report", func(w http.ResponseWriter, req *http.Request) {
		auth.RequireAuth(http.HandlerFunc(handlers.GenerateReport)).ServeHTTP(w, req)
	})
	r.Get("/api/admin", func(w http.ResponseWriter, req *http.Request) {
		auth.RequireAuth(http.HandlerFunc(handlers.AdminDashboard)).ServeHTTP(w, req)
	})

	// worker proxy: scrape/ingest
	r.Post("/api/ingest/url", func(w http.ResponseWriter, req *http.Request) {
		auth.RequireAuth(http.HandlerFunc(handlers.IngestURL)).ServeHTTP(w, req)
	})

	// AI endpoints (optional)
	r.Post("/api/ai/chat", handlers.ChatHandler)
	r.Post("/api/ai/embed", handlers.EmbedHandler)

	// static frontend (optional)
	r.Handle("/*", http.FileServer(http.Dir("./web")))

	port := os.Getenv("API_PORT")
	if port == "" { port = "8080" }
	log.Printf("🌍 API Gateway running on :%s", port)
	log.Fatal(http.ListenAndServe(":"+port, r))
}

func mustAtoiEnv(key string, def int) int {
	if v := os.Getenv(key); v != "" {
		if n, err := atoi(v); err == nil { return n }
	}
	return def
}

func atoi(s string) (int, error) {
	var n, sign int = 0, 1
	for i, r := range s {
		if i == 0 && r == '-' { sign = -1; continue }
		if r < '0' || r > '9' { return 0, &atoiErr{s} }
		n = n*10 + int(r-'0')
	}
	return sign * n, nil
}
type atoiErr struct{ s string }
func (e *atoiErr) Error() string { return "invalid int: " + e.s }

// lightweight .env loader (KEY=VALUE lines, ignores comments)
func loadDotEnv(path string) error {
	b, err := os.ReadFile(path)
	if err != nil { return err }
	for _, line := range splitLines(string(b)) {
		if line == "" || line[0] == '#' { continue }
		kv := splitOnce(line, '=')
		if len(kv) != 2 { continue }
		if os.Getenv(kv[0]) == "" {
			_ = os.Setenv(kv[0], kv[1])
		}
	}
	return nil
}
func splitLines(s string) []string {
	out := []string{}
	start := 0
	for i, r := range s {
		if r == '\n' || r == '\r' {
			if i > start { out = append(out, s[start:i]) }
			start = i + 1
		}
	}
	if start < len(s) { out = append(out, s[start:]) }
	return out
}
func splitOnce(s string, sep rune) []string {
	for i, r := range s {
		if r == sep { return []string{s[:i], s[i+1:]} }
	}
	return []string{s}
}