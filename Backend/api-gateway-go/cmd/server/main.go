package main

import (
	"log"
	"net/http"
	"os"

	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"

	"codered/api-gateway-go/internal/auth"
	"codered/api-gateway-go/internal/cache"
	"codered/api-gateway-go/internal/db"
	"codered/api-gateway-go/internal/handlers"
	"codered/api-gateway-go/internal/security"
)

func main() {
	db.Init()
	cache.Init()

	r := chi.NewRouter()
	r.Use(middleware.Logger)
	r.Use(security.CSRFMiddleware)
	r.Use(security.RateLimitMiddleware)

	// Authentication routes
	r.Post("/api/auth/login", auth.Login)
	r.Post("/api/auth/logout", auth.Logout)
	r.Get("/api/auth/me", auth.RequireAuth(http.HandlerFunc(auth.Me)))

	// Report generation routes
	r.Post("/api/report", auth.RequireAuth(http.HandlerFunc(handlers.GenerateReport)))

	// Admin dashboard
	r.Get("/api/admin", auth.RequireAuth(http.HandlerFunc(handlers.AdminDashboard)))

	// Serve static frontend (optional)
	r.Handle("/*", http.FileServer(http.Dir("./web")))

	port := os.Getenv("API_PORT")
	if port == "" {
		port = "8080"
	}
	log.Printf("🌐 API Gateway running on :%s", port)
	http.ListenAndServe(":"+port, r)
}