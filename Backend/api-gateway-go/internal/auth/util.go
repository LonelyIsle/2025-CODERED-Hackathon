package auth

import (
	"crypto/rand"
	"encoding/base64"
	"net/http"
	"time"

	"golang.org/x/crypto/bcrypt"
)

const sessionCookieName = "sess"

// session cookie helper
func sessionCookie(token string, ttl time.Duration) *http.Cookie {
	return &http.Cookie{
		Name:     sessionCookieName,
		Value:    token,
		Path:     "/",
		HttpOnly: true,
		SameSite: http.SameSiteLaxMode,
		MaxAge:   int(ttl.Seconds()),
	}
}

func newToken(nBytes int) (string, error) {
	if nBytes <= 0 {
		nBytes = 32
	}
	b := make([]byte, nBytes)
	if _, err := rand.Read(b); err != nil {
		return "", err
	}
	return base64.RawURLEncoding.EncodeToString(b), nil
}

// VerifyPassword compares plaintext with a bcrypt hash.
func VerifyPassword(plain, bcryptHash string) bool {
	if bcryptHash == "" {
		return false
	}
	return bcrypt.CompareHashAndPassword([]byte(bcryptHash), []byte(plain)) == nil
}
