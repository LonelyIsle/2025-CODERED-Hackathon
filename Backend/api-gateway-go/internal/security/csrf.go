package security

import (
	"crypto/rand"
	"encoding/base64"
	"net/http"
)

func CSRFMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method == http.MethodGet {
			next.ServeHTTP(w, r)
			return
		}
		token := r.Header.Get("X-CSRF-Token")
		cookie, _ := r.Cookie("csrf")
		if cookie == nil || cookie.Value != token {
			http.Error(w, "CSRF validation failed", http.StatusForbidden)
			return
		}
		next.ServeHTTP(w, r)
	})
}

func IssueCSRFToken(w http.ResponseWriter) string {
	b := make([]byte, 16)
	rand.Read(b)
	token := base64.StdEncoding.EncodeToString(b)
	http.SetCookie(w, &http.Cookie{
		Name:     "csrf",
		Value:    token,
		Path:     "/",
		HttpOnly: false,
	})
	return token
}