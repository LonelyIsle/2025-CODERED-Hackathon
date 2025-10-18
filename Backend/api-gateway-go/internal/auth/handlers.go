package auth

import (
	"context"
	"encoding/json"
	"net/http"
	"time"

	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/cache"
	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/db"
)

func Login(w http.ResponseWriter, r *http.Request) {
	var body struct {
		Username string `json:"username"`
		Password string `json:"password"`
	}
	json.NewDecoder(r.Body).Decode(&body)

	u, err := db.GetUserByUsername(context.Background(), body.Username)
	if err != nil || !VerifyPassword(body.Password, u.Password) {
		http.Error(w, "invalid credentials", http.StatusUnauthorized)
		return
	}

	token := RandString(32)
	cache.Set("sess:"+token, body.Username, time.Hour*24)
	http.SetCookie(w, &http.Cookie{
		Name:     "session",
		Value:    token,
		Path:     "/",
		HttpOnly: true,
		Secure:   true,
		SameSite: http.SameSiteStrictMode,
	})
	w.WriteHeader(http.StatusOK)
}

func Logout(w http.ResponseWriter, r *http.Request) {
	c, err := r.Cookie("session")
	if err == nil {
		cache.Delete("sess:" + c.Value)
	}
	http.SetCookie(w, &http.Cookie{Name: "session", Value: "", Path: "/", MaxAge: -1})
	w.WriteHeader(http.StatusOK)
}

func Me(w http.ResponseWriter, r *http.Request) {
	user := r.Context().Value("user").(string)
	json.NewEncoder(w).Encode(map[string]string{"user": user})
}

func RandString(n int) string {
	const chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
	b := make([]byte, n)
	rand.Read(b)
	for i := range b {
		b[i] = chars[int(b[i])%len(chars)]
	}
	return string(b)
}