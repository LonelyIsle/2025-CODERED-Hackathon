package security

import (
	"crypto/rand"
	"crypto/subtle"
	"encoding/base64"
	"net/http"
	"strings"
)

// isSafeMethod returns true for "safe" HTTP methods that shouldn't require CSRF checks.
func isSafeMethod(m string) bool {
	switch m {
	case http.MethodGet, http.MethodHead, http.MethodOptions:
		return true
	default:
		return false
	}
}

// CSRFMiddleware enforces a simple "double submit cookie" strategy:
//  - client first hits /api/auth/csrf to receive a "csrf" cookie and JSON token
//  - subsequent state-changing requests must echo the token in X-CSRF-Token header
//  - header must match the cookie value
func CSRFMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// Allow safe methods and the token-issuing endpoint to pass through
		if isSafeMethod(r.Method) || strings.HasSuffix(r.URL.Path, "/api/auth/csrf") {
			next.ServeHTTP(w, r)
			return
		}

		token := r.Header.Get("X-CSRF-Token")
		cookie, err := r.Cookie("csrf")
		if err != nil || cookie == nil || token == "" {
			http.Error(w, "CSRF validation failed", http.StatusForbidden)
			return
		}

		// Constant-time compare to avoid timing side-channels
		if subtle.ConstantTimeCompare([]byte(cookie.Value), []byte(token)) != 1 {
			http.Error(w, "CSRF validation failed", http.StatusForbidden)
			return
		}

		next.ServeHTTP(w, r)
	})
}

// IssueCSRFToken generates a URL-safe token, sets it as a cookie, and returns it
// so the handler can also include it in the JSON response.
func IssueCSRFToken(w http.ResponseWriter) string {
	// 32 bytes -> 43 chars with RawURLEncoding (no '+' '/' or '=')
	buf := make([]byte, 32)
	_, _ = rand.Read(buf)
	token := base64.RawURLEncoding.EncodeToString(buf)

	http.SetCookie(w, &http.Cookie{
		Name:     "csrf",
		Value:    token,
		Path:     "/",
		HttpOnly: false,                 // frontend JS must read & echo it
		SameSite: http.SameSiteLaxMode,  // helps mitigate CSRF
		Secure:   false,                 // set true when served via HTTPS/Cloudflare
		// MaxAge/Expires can be added if you want a finite lifetime
	})

	return token
}