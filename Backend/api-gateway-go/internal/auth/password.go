package auth

import (
	"crypto/rand"
	"crypto/subtle"
	"encoding/base64"
	"strings"

	"golang.org/x/crypto/argon2"
)

// HashPassword returns "saltB64$hashB64" using Argon2id with params:
// iterations=1, memory=64MB, parallelism=4, keyLen=32.
func HashPassword(password string) string {
	salt := make([]byte, 16)
	_, _ = rand.Read(salt) // ignore error for now; extremely unlikely to fail

	hash := argon2.IDKey([]byte(password), salt, 1, 64*1024, 4, 32)

	return base64.StdEncoding.EncodeToString(salt) + "$" + base64.StdEncoding.EncodeToString(hash)
}

// VerifyPassword supports two storage formats:
//
// 1) Argon2id "saltB64$hashB64"  -> recompute and constant-time compare the raw bytes.
// 2) Plaintext (no "$")          -> accepted for dev/bootstrap; not recommended for production.
func VerifyPassword(password, stored string) bool {
	// If it looks like an Argon2id value, verify with Argon2id.
	if strings.Contains(stored, "$") {
		parts := strings.SplitN(stored, "$", 2)
		if len(parts) != 2 {
			return false
		}

		salt, err := base64.StdEncoding.DecodeString(parts[0])
		if err != nil {
			return false
		}
		wantHash, err := base64.StdEncoding.DecodeString(parts[1])
		if err != nil {
			return false
		}

		gotHash := argon2.IDKey([]byte(password), salt, 1, 64*1024, 4, 32)
		return subtle.ConstantTimeCompare(gotHash, wantHash) == 1
	}

	// Otherwise, treat the stored value as plaintext (constant-time compare).
	return subtle.ConstantTimeCompare([]byte(password), []byte(stored)) == 1
}