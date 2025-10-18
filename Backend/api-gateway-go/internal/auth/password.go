package auth

import (
	"crypto/rand"
	"encoding/base64"
	"strings"

	"golang.org/x/crypto/argon2"
)

func HashPassword(password string) string {
	salt := make([]byte, 16)
	rand.Read(salt)
	hash := argon2.IDKey([]byte(password), salt, 1, 64*1024, 4, 32)
	return base64.StdEncoding.EncodeToString(salt) + "$" + base64.StdEncoding.EncodeToString(hash)
}

func VerifyPassword(password, encoded string) bool {
	parts := strings.Split(encoded, "$")
	if len(parts) != 2 {
		return false
	}
	salt, _ := base64.StdEncoding.DecodeString(parts[0])
	hash := argon2.IDKey([]byte(password), salt, 1, 64*1024, 4, 32)
	return base64.StdEncoding.EncodeToString(hash) == parts[1]
}