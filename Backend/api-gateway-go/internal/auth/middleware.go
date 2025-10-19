package auth

import (
	"net/http"

	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/cache"
)

const sessionCookie = "session"

func RequireAuth(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		c, err := r.Cookie(sessionCookie)
		if err != nil || c.Value == "" {
			http.Error(w, "unauthorized", http.StatusUnauthorized)
			return
		}
		if _, err := cache.Get("sess:" + c.Value); err != nil {
			http.Error(w, "unauthorized", http.StatusUnauthorized)
			return
		}
		next.ServeHTTP(w, r)
	})
}