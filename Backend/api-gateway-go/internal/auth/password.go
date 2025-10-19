package auth

import (
	"crypto/rand"
	"encoding/base64"
	"errors"
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
	if _, err := rand.Read(salt); err != nil {
		return "", err
	}
	sum := argon2.IDKey([]byte(plain), salt, p.Iterations, p.Memory, p.Parallelism, p.KeyLen)
	enc := "argon2id$" +
		strconv.Itoa(int(p.Memory)) + "$" +
		strconv.Itoa(int(p.Iterations)) + "$" +
		strconv.Itoa(int(p.Parallelism)) + "$" +
		base64.RawStdEncoding.EncodeToString(salt) + "$" +
		base64.RawStdEncoding.EncodeToString(sum)
	return enc, nil
}

func VerifyPassword(plain, encoded string) (bool, error) {
	var mem, iters, par int
	var saltB64, sumB64 string
	_, err := fmtSscanf(encoded, "argon2id$%d$%d$%d$%s$%s", &mem, &iters, &par, &saltB64, &sumB64)
	if err != nil {
		return false, errors.New("bad hash format")
	}
	salt, err := base64.RawStdEncoding.DecodeString(saltB64)
	if err != nil {
		return false, err
	}
	sum, err := base64.RawStdEncoding.DecodeString(sumB64)
	if err != nil {
		return false, err
	}
	keyLen := uint32(len(sum))
	out := argon2.IDKey([]byte(plain), salt, uint32(iters), uint32(mem), uint8(par), keyLen)
	if len(out) != len(sum) {
		return false, nil
	}
	// constant-time compare
	var diff uint8
	for i := range out {
		diff |= out[i] ^ sum[i]
	}
	return diff == 0, nil
}

// tiny wrapper so we don't import fmt just for Sscanf
func fmtSscanf(s, f string, a ...any) (int, error) {
	type scan interface {
		Scan(string, string, ...any) (int, error)
	}
	var _scan scan = (fmtScan)(0)
	return _scan.Scan(s, f, a...)
}
type fmtScan int
func (fmtScan) Scan(s, f string, a ...any) (int, error) {
	return fmtSscanfReal(s, f, a...)
}

//go:build !js
// +build !js

// split into separate function to avoid linter noise
func fmtSscanfReal(s, f string, a ...any) (int, error) {
	return fmtSscanfStd(s, f, a...)
}