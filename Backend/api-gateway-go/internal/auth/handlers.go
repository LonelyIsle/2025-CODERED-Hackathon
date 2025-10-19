package auth

import (
	"encoding/json"
	"net/http"
	"os"
	"time"

	"github.com/google/uuid"

	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/cache"
	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/db"
)

type loginBody struct {
	Email    string `json:"email"`
	Password string `json:"password"`
}

func Login(w http.ResponseWriter, r *http.Request) {
	var body loginBody
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		http.Error(w, "bad json", http.StatusBadRequest)
		return
	}
	u, err := db.GetUserByEmail(r.Context(), body.Email)
	if err != nil {
		http.Error(w, "invalid credentials", http.StatusUnauthorized)
		return
	}
	ok, err := VerifyPassword(body.Password, u.PasswordHash)
	if err != nil || !ok {
		http.Error(w, "invalid credentials", http.StatusUnauthorized)
		return
	}

	ttl := 24 * time.Hour
	if v := os.Getenv("SESSION_TTL_HOURS"); v != "" {
		if n, _ := time.ParseDuration(v + "h"); n > 0 { ttl = n }
	}
	token := uuid.NewString()
	if err := cache.Set("sess:"+token, u.Email, ttl); err != nil {
		http.Error(w, "session store failed", http.StatusInternalServerError)
		return
	}

	http.SetCookie(w, &http.Cookie{
		Name:     sessionCookie,
		Value:    token,
		Path:     "/",
		HttpOnly: true,
		SameSite: http.SameSiteLaxMode,
	})
	w.Header().Set("Content-Type", "application/json")
	_, _ = w.Write([]byte(`{"ok":true}`))
}

func Logout(w http.ResponseWriter, r *http.Request) {
	if c, err := r.Cookie(sessionCookie); err == nil && c.Value != "" {
		_ = cache.Delete("sess:" + c.Value)
	}
	http.SetCookie(w, &http.Cookie{
		Name:     sessionCookie,
		Value:    "",
		Path:     "/",
		MaxAge:   -1,
		HttpOnly: true,
		SameSite: http.SameSiteLaxMode,
	})
	w.Header().Set("Content-Type", "application/json")
	_, _ = w.Write([]byte(`{"ok":true}`))
}

func Me(w http.ResponseWriter, r *http.Request) {
	c, err := r.Cookie(sessionCookie)
	if err != nil || c.Value == "" {
		http.Error(w, "unauthorized", http.StatusUnauthorized); return
	}
	email, err := cache.Get("sess:" + c.Value)
	if err != nil {
		http.Error(w, "unauthorized", http.StatusUnauthorized); return
	}
	_ = json.NewEncoder(w).Encode(map[string]any{"ok": true, "email": email})
}