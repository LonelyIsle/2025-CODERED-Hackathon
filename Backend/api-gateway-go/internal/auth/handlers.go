package auth

import (
	"context"
	"encoding/json"
	"errors"
	"log"
	"net/http"
	"os"
	"strings"
	"time"

	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/cache"
	"github.com/LonelyIsle/2025-CODERED-Hackathon/Backend/api-gateway-go/internal/db"
)

// --- Helpers ----------------------------------------------------------------

func allowPlaintext() bool { return strings.EqualFold(os.Getenv("ALLOW_PLAINTEXT_PASSWORDS"), "true") }
func isProd() bool         { return strings.EqualFold(os.Getenv("APP_ENV"), "production") }

func writeJSON(w http.ResponseWriter, code int, v any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(code)
	_ = json.NewEncoder(w).Encode(v)
}

// RandString: simple non-crypto session token (fine for local dev)
func RandString(n int) string {
	const chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
	b := make([]byte, n)
	// pseudo-random is OK for dev; swap to crypto/rand if you want stronger tokens
	for i := range b {
		b[i] = chars[time.Now().UnixNano()%int64(len(chars))]
	}
	return string(b)
}

// --- Handlers ---------------------------------------------------------------

func Login(w http.ResponseWriter, r *http.Request) {
	var body struct {
		Email    string `json:"email"`
		Password string `json:"password"`
	}
	dec := json.NewDecoder(r.Body)
	dec.DisallowUnknownFields()
	if err := dec.Decode(&body); err != nil {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "invalid json"})
		return
	}
	body.Email = strings.TrimSpace(strings.ToLower(body.Email))
	if body.Email == "" || body.Password == "" {
		writeJSON(w, http.StatusBadRequest, map[string]string{"error": "email and password required"})
		return
	}

	// NOTE: your DB has email & plaintext password in password_hash column
	u, err := db.GetUserByEmail(context.Background(), body.Email)
	if err != nil {
		log.Printf("login: user fetch error for %s: %v", body.Email, err)
		http.Error(w, "invalid credentials", http.StatusUnauthorized)
		return
	}

	// Password check: allow plaintext in dev, otherwise use VerifyPassword (argon2)
	ok := false
	if allowPlaintext() {
		ok = (u.Password == body.Password || u.PasswordHash == body.Password)
	} else {
		// If your user struct uses PasswordHash, prefer that.
		ph := u.PasswordHash
		if ph == "" {
			ph = u.Password // fallback if your struct uses Password for the hash
		}
		ok = VerifyPassword(body.Password, ph)
	}
	if !ok {
		http.Error(w, "invalid credentials", http.StatusUnauthorized)
		return
	}

	// Create session
	token := RandString(32)

	// Store identity in cache for 24h
	if err := cache.Set("sess:"+token, body.Email, 24*time.Hour); err != nil {
		log.Printf("login: cache set error: %v", err)
		writeJSON(w, http.StatusInternalServerError, map[string]string{"error": "session error"})
		return
	}

	// Cookie: Secure=false for local HTTP; true only if prod+TLS
	secure := false
	if isProd() && r.TLS != nil {
		secure = true
	}
	http.SetCookie(w, &http.Cookie{
		Name:     "session",
		Value:    token,
		Path:     "/",
		HttpOnly: true,
		SameSite: http.SameSiteLaxMode,
		Secure:   secure,
		// MaxAge not required since we also expire in cache, but you can add it if desired
	})

	writeJSON(w, http.StatusOK, map[string]any{
		"ok":    true,
		"email": body.Email,
		"role":  u.Role,
	})
}

func Logout(w http.ResponseWriter, r *http.Request) {
	if c, err := r.Cookie("session"); err == nil && c.Value != "" {
		if err := cache.Delete("sess:" + c.Value); err != nil {
			log.Printf("logout: cache delete error: %v", err)
		}
	}
	// clear cookie
	http.SetCookie(w, &http.Cookie{
		Name:   "session",
		Value:  "",
		Path:   "/",
		MaxAge: -1,
	})
	writeJSON(w, http.StatusOK, map[string]any{"ok": true})
}

func Me(w http.ResponseWriter, r *http.Request) {
	// Expect your RequireAuth middleware to set context value "user" (string email)
	v := r.Context().Value("user")
	email, _ := v.(string)
	if email == "" {
		writeJSON(w, http.StatusUnauthorized, map[string]string{"error": "unauthorized"})
		return
	}
	writeJSON(w, http.StatusOK, map[string]string{"email": email})
}

// --- Middleware (if you don’t already have it) ------------------------------
// If your existing auth.RequireAuth middleware already pulls from the session
// cookie and Valkey, keep that one. This is a reference implementation.

func RequireAuth(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		c, err := r.Cookie("session")
		if err != nil || c.Value == "" {
			http.Error(w, "unauthorized", http.StatusUnauthorized)
			return
		}
		email, err := cache.Get("sess:" + c.Value)
		if err != nil || email == "" {
			http.Error(w, "unauthorized", http.StatusUnauthorized)
			return
		}
		ctx := context.WithValue(r.Context(), "user", email)
		next.ServeHTTP(w, r.WithContext(ctx))
	})
}

// --- VerifyPassword ---------------------------------------------------------
// Keep your existing Argon2-based implementation for non-plaintext mode.
// This stub shows the signature the Login() code expects.

func VerifyPassword(password, encoded string) bool {
	// Your real implementation goes here; this keeps the signature.
	// Return false if the stored format is not an argon2 pair.
	_ = password
	_ = encoded
	return false
}

// --- DB glue expectations ---------------------------------------------------
// Your db package should expose something like:
//
// func GetUserByEmail(ctx context.Context, email string) (*User, error) {
//     // SELECT id, email, password_hash, role FROM users WHERE email=$1
// }
//
// type User struct {
//     ID           int64
//     Email        string
//     Password     string      // if you used this name
//     PasswordHash string      // or this, depending on your struct
//     Role         string
// }
//
// Return (nil, error) if not found.
func _docOnlyAvoidUnused() error {
	_ = errors.New
	return nil
}