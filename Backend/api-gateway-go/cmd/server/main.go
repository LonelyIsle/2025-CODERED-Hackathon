package main

import (
	"encoding/json"
	"log"
	"net/http"
	"os"
	"time"

	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	"github.com/joho/godotenv"

	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/auth"
	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/cache"
	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/db"
	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/handlers"
	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/security"
)

// --- CSRF helpers (public endpoint) ------------------------------------------------------

func newCSRFToken() string {
	const letters = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
	b := make([]byte, 32)
	for i := range b {
		b[i] = letters[time.Now().UnixNano()%int64(len(letters))]
	}
	return string(b)
}

func csrfHandler(w http.ResponseWriter, r *http.Request) {
	token := newCSRFToken()
	http.SetCookie(w, &http.Cookie{
		Name:     "csrf",
		Value:    token,
		Path:     "/",
		HttpOnly: false, // frontend JS must echo it in X-CSRF-Token
		SameSite: http.SameSiteLaxMode,
		Secure:   false, // set true when served behind HTTPS
	})
	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(map[string]string{"csrf": token})
}

// --- tiny sanity route -------------------------------------------------------------------

func pingHandler(w http.ResponseWriter, r *http.Request) {
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write([]byte("pong"))
}

func main() {
	// Load env (works when running from api-gateway-go or repo root)
	_ = godotenv.Load(".env")
	_ = godotenv.Load("../.env") // fallback if binary runs from /api-gateway-go

	// init subsystems
	db.Init()
	cache.Init()

	r := chi.NewRouter()
	r.Use(middleware.RequestID)
	r.Use(middleware.RealIP)
	r.Use(middleware.Logger)
	r.Use(middleware.Recoverer)

// ---------- Public routes (NO CSRF) ----------
r.Route("/api/auth", func(pub chi.Router) {
    pub.Get("/csrf", csrfHandler)
})

// ---------- Protected API (CSRF + rate limit) ----------
r.Route("/api", func(api chi.Router) {
    // Allow turning off CSRF locally: export DISABLE_CSRF=true
    if os.Getenv("DISABLE_CSRF") != "true" {
        api.Use(security.CSRFMiddleware)
    }
    api.Use(security.RateLimitMiddleware)

    // health/ping
    api.Get("/ping", pingHandler)

    // Auth-only endpoints
    api.Route("/auth", func(a chi.Router) {
        a.Post("/login", auth.Login)   // expects X-CSRF-Token + csrf cookie
        a.Post("/logout", auth.Logout)
        a.Get("/me", func(w http.ResponseWriter, req *http.Request) {
            auth.RequireAuth(http.HandlerFunc(auth.Me)).ServeHTTP(w, req)
        })
    })

    // AI endpoints (protect or not—your call; here they’re protected)
    api.Post("/ai/chat", func(w http.ResponseWriter, req *http.Request) {
        auth.RequireAuth(http.HandlerFunc(handlers.ChatHandler)).ServeHTTP(w, req)
    })
    api.Post("/ai/embed", func(w http.ResponseWriter, req *http.Request) {
        auth.RequireAuth(http.HandlerFunc(handlers.EmbedHandler)).ServeHTTP(w, req)
    })

    // Reports/Admin
    api.Post("/report", func(w http.ResponseWriter, req *http.Request) {
        auth.RequireAuth(http.HandlerFunc(handlers.GenerateReport)).ServeHTTP(w, req)
    })
    api.Get("/admin", func(w http.ResponseWriter, req *http.Request) {
        auth.RequireAuth(http.HandlerFunc(handlers.AdminDashboard)).ServeHTTP(w, req)
    })
    api.Post("/admin/ingest", func(w http.ResponseWriter, req *http.Request) {
        auth.RequireAuth(http.HandlerFunc(handlers.IngestURL)).ServeHTTP(w, req)
    })
})

	// (Optional) static fallback if you ever serve frontend from the Go binary.
	// Nginx is serving your dist already, so you can remove this if unused.
	// r.Handle("/*", http.FileServer(http.Dir("./web")))

	port := os.Getenv("API_PORT")
	if port == "" {
		port = "8081"
	}

	// 👉 Dump all routes on startup so you can verify /api/auth/login exists
    if err := chi.Walk(r, func(method, route string, _ http.Handler, _ ...func(http.Handler) http.Handler) error {
        log.Printf("route %-6s %s", method, route)
        return nil
    }); err != nil {
        log.Printf("chi.Walk error: %v", err)
    }

	log.Printf("🌍 API Gateway running on :%s", port)
	if err := http.ListenAndServe(":"+port, r); err != nil {
		log.Fatalf("❌ Failed to start server: %v", err)

	}
}