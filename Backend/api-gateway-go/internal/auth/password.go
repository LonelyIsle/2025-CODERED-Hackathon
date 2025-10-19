package auth

import (
	"crypto/rand"
	"encoding/base64"
	"errors"
	"fmt"
	"os"
	"strconv"

	"golang.org/x/crypto/argon2"
)

type argonParams struct {
	Memory      uint32
	Iterations  uint32
	Parallelism uint8
	KeyLen      uint32
}

func envU32(key string, def uint32) uint32 {
	if s := os.Getenv(key); s != "" {
		if v, err := strconv.ParseUint(s, 10, 32); err == nil {
			return uint32(v)
		}
	}
	return def
}

func paramsFromEnv() argonParams {
	return argonParams{
		Memory:      envU32("PASSWORD_HASH_MEMORY_KB", 64*1024),
		Iterations:  envU32("PASSWORD_HASH_ITERATIONS", 1),
		Parallelism: uint8(envU32("PASSWORD_HASH_PARALLELISM", 4)),
		KeyLen:      envU32("PASSWORD_HASH_KEYLEN", 32),
	}
}

func HashPassword(plain string) (string, error) {
	p := paramsFromEnv()
	salt := make([]byte, 16)
	if _, err := rand.Read(salt); err != nil { return "", err }
	sum := argon2.IDKey([]byte(plain), salt, p.Iterations, p.Memory, p.Parallelism, p.KeyLen)
	enc := fmt.Sprintf("argon2id$%d$%d$%d$%s$%s",
		p.Memory, p.Iterations, p.Parallelism,
		base64.RawStdEncoding.EncodeToString(salt),
		base64.RawStdEncoding.EncodeToString(sum),
	)
	return enc, nil
}

func VerifyPassword(plain, encoded string) (bool, error) {
	var mem, iters, par int
	var saltB64, sumB64 string
	if _, err := fmt.Sscanf(encoded, "argon2id$%d$%d$%d$%s$%s", &mem, &iters, &par, &saltB64, &sumB64); err != nil {
		return false, errors.New("bad hash format")
	}
	salt, err := base64.RawStdEncoding.DecodeString(saltB64)
	if err != nil { return false, err }
	sum, err := base64.RawStdEncoding.DecodeString(sumB64)
	if err != nil { return false, err }
	out := argon2.IDKey([]byte(plain), salt, uint32(iters), uint32(mem), uint8(par), uint32(len(sum)))
	if len(out) != len(sum) { return false, nil }
	var diff uint8
	for i := range out { diff |= out[i] ^ sum[i] }
	return diff == 0, nil
}