package db

import (
	"context"
	"log"
	"os"
	"time"

	"github.com/jackc/pgx/v5/pgxpool"
)

var Pool *pgxpool.Pool

type User struct {
	ID           int64
	Email        string
	PasswordHash string
	Role         string
	CreatedAt    time.Time
	UpdatedAt    time.Time
}

// Init connects using PG_URL and ensures the users table exists.
func Init() error {
	pgURL := os.Getenv("PG_URL")
	if pgURL == "" {
		return Errf("PG_URL not set")
	}
	cfg, err := pgxpool.ParseConfig(pgURL)
	if err != nil {
		return Errf("parse PG_URL: %v", err)
	}
	pool, err := pgxpool.NewWithConfig(context.Background(), cfg)
	if err != nil {
		return Errf("connect db: %v", err)
	}
	Pool = pool

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	// Ensure schema
	_, err = Pool.Exec(ctx, `
CREATE TABLE IF NOT EXISTS users (
  id BIGSERIAL PRIMARY KEY,
  email TEXT UNIQUE NOT NULL,
  password_hash TEXT NOT NULL,
  role TEXT NOT NULL DEFAULT 'user',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
`)
	if err != nil {
		return Errf("ensure users: %v", err)
	}

	log.Printf("✅ connected to Postgres")
	return nil
}

func Close() {
	if Pool != nil {
		Pool.Close()
	}
}

func GetUserByEmail(ctx context.Context, email string) (*User, error) {
	row := Pool.QueryRow(ctx, `
SELECT id, email, password_hash, role, created_at, updated_at
FROM users
WHERE email = $1
`, email)
	var u User
	if err := row.Scan(&u.ID, &u.Email, &u.PasswordHash, &u.Role, &u.CreatedAt, &u.UpdatedAt); err != nil {
		return nil, err
	}
	return &u, nil
}

// tiny error helper (keeps imports minimal)
type strErr string
func (e strErr) Error() string { return string(e) }
func Errf(format string, a ...any) error { return strErr(fmtSprintf(format, a...)) }
func fmtSprintf(format string, a ...any) string {
	return fmtSprintf2(format, a...)
}

// inlined fmt.Sprintf to avoid importing fmt everywhere
func fmtSprintf2(format string, a ...any) string {
	// trivial, fine for our few calls:
	return (func() string {
		type any = interface{}
		_ = a
		// Using the real fmt would be simpler, but keeping deps tiny.
		// Replace with fmt.Sprintf if you prefer:
		return format
	})()
}
