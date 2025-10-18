package main

import (
	"log"
	"net/http"
	"os"

	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	"github.com/joho/godotenv"

	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/auth"
	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/cache"
	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/db"
	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/handlers"
	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/security"
)

func main() {
	// ==============================================
	// 🔧 Load environment variables
	// ==============================================
	if err := godotenv.Load("../../.env"); err != nil {
		log.Println("⚠️  No .env file found, relying on system environment variables")
	}

	// ==============================================
	// 🧠 Initialize core subsystems
	// ==============================================
	db.Init()
	cache.Init()

	// ==============================================
	// 🚦 Setup Router
	// ==============================================
	r := chi.NewRouter()
	r.Use(middleware.Logger)
	r.Use(security.CSRFMiddleware)
	r.Use(security.RateLimitMiddleware)

	// ==============================================
	// 🔐 Authentication routes
	// ==============================================
	r.Post("/api/auth/login", auth.Login)
	r.Post("/api/auth/logout", auth.Logout)
	r.Get("/api/auth/me", auth.RequireAuth(http.HandlerFunc(auth.Me)))

	// ==============================================
	// 📊 Report routes
	// ==============================================
	r.Post("/api/report", auth.RequireAuth(http.HandlerFunc(handlers.GenerateReport)))

	// ==============================================
	// 🧰 Admin routes
	// ==============================================
	r.Get("/api/admin", auth.RequireAuth(http.HandlerFunc(handlers.AdminDashboard)))

	// ==============================================
	// 🌐 Serve static frontend (optional)
	// ==============================================
	r.Handle("/*", http.FileServer(http.Dir("./web")))

	// ==============================================
	// 🚀 Start server
	// ==============================================
	port := os.Getenv("API_PORT")
	if port == "" {
		port = "8080"
	}
	log.Printf("🌍 API Gateway running on :%s", port)
	if err := http.ListenAndServe(":"+port, r); err != nil {
		log.Fatalf("❌ Failed to start server: %v", err)
	}
}