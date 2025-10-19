package auth

import (
	"context"
	"encoding/json"
	"net/http"
	"os"
	"time"

	"github.com/google/uuid"

	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/cache"
	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/db"
)

type loginReq struct {
	Email    string `json:"email"`
	Password string `json:"password"`
}

func Login(w http.ResponseWriter, r *http.Request) {
	var body loginReq
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		http.Error(w, "bad request", http.StatusBadRequest)
		return
	}
	ctx, cancel := context.WithTimeout(r.Context(), 5*time.Second)
	defer cancel()

	u, err := db.GetUserByEmail(ctx, body.Email)
	if err != nil || !VerifyPassword(body.Password, u.PasswordHash) {
		http.Error(w, "invalid credentials", http.StatusUnauthorized)
		return
	}
	ttlHours := 24
	if v := os.Getenv("SESSION_TTL_HOURS"); v != "" {
		// ignore parse errors; default 24h
	}
	ttl := time.Duration(ttlHours) * time.Hour

	token := uuid.NewString()
	if err := cache.Set("sess:"+token, u.Email, ttl); err != nil {
		http.Error(w, "session error", http.StatusInternalServerError)
		return
	}
	http.SetCookie(w, sessionCookie(token, ttl))
	w.Header().Set("Content-Type", "application/json")
	_, _ = w.Write([]byte(`{"ok":true}`))
}

func Logout(w http.ResponseWriter, r *http.Request) {
	c, err := r.Cookie(sessionCookieName)
	if err == nil && c.Value != "" {
		_ = cache.Delete("sess:" + c.Value)
	}
	// expire client cookie
	exp := sessionCookie("", time.Second)
	exp.MaxAge = -1
	http.SetCookie(w, exp)
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write([]byte(`{"ok":true}`))
}

func RequireAuth(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		c, err := r.Cookie(sessionCookieName)
		if err != nil || c.Value == "" {
			http.Error(w, "unauthorized", http.StatusUnauthorized)
			return
		}
		email, err := cache.Get("sess:" + c.Value)
		if err != nil || email == "" {
			http.Error(w, "unauthorized", http.StatusUnauthorized)
			return
		}
		// if you want, put email into context here
		next.ServeHTTP(w, r)
	})
}

func Me(w http.ResponseWriter, r *http.Request) {
	c, err := r.Cookie(sessionCookieName)
	if err != nil || c.Value == "" {
		http.Error(w, "unauthorized", http.StatusUnauthorized)
		return
	}
	email, err := cache.Get("sess:" + c.Value)
	if err != nil || email == "" {
		http.Error(w, "unauthorized", http.StatusUnauthorized)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	_, _ = w.Write([]byte(`{"ok":true,"email":"` + email + `"}`))
}
