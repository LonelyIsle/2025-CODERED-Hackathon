package security

import (
	"crypto/rand"
	"crypto/subtle"
	"encoding/base64"
	"net/http"
	"os"
	"strings"
)

func isSafeMethod(m string) bool {
	return m == http.MethodGet || m == http.MethodHead || m == http.MethodOptions
}

// CSRFMiddleware: if DISABLE_CSRF=true, it's a no-op.
// Otherwise, for non-safe methods it requires X-CSRF-Token to match the "csrf" cookie.
// The token can be obtained from GET /api/auth/csrf.
func CSRFMiddleware(next http.Handler) http.Handler {
	disabled := os.Getenv("DISABLE_CSRF") == "true"
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if disabled || isSafeMethod(r.Method) || strings.HasSuffix(r.URL.Path, "/api/auth/csrf") {
			next.ServeHTTP(w, r)
			return
		}
		header := r.Header.Get("X-CSRF-Token")
		cookie, err := r.Cookie("csrf")
		if err != nil || header == "" || cookie == nil {
			http.Error(w, "CSRF validation failed", http.StatusForbidden)
			return
		}
		if subtle.ConstantTimeCompare([]byte(cookie.Value), []byte(header)) != 1 {
			http.Error(w, "CSRF validation failed", http.StatusForbidden)
			return
		}
		next.ServeHTTP(w, r)
	})
}

// IssueCSRFToken sets a cookie and returns the token for JSON responses.
func IssueCSRFToken(w http.ResponseWriter) string {
	buf := make([]byte, 32)
	_, _ = rand.Read(buf)
	token := base64.RawURLEncoding.EncodeToString(buf)
	http.SetCookie(w, &http.Cookie{
		Name:     "csrf",
		Value:    token,
		Path:     "/",
		HttpOnly: false,
		SameSite: http.SameSiteLaxMode,
		Secure:   false, // set true behind HTTPS (e.g., Cloudflare)
	})
	return token
}