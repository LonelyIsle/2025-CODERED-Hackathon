package db

import (
	"context"
	"database/sql"
	"os"
	"time"

	_ "github.com/jackc/pgx/v5/stdlib"
)

var defaultDB *sql.DB

func Init() error {
	dsn := os.Getenv("PG_URL")
	if dsn == "" {
		dsn = "postgres://climate:climate123@127.0.0.1:5432/codered?sslmode=disable"
	}
	db, err := sql.Open("pgx", dsn)
	if err != nil {
		return err
	}
	db.SetMaxIdleConns(4)
	db.SetMaxOpenConns(16)
	db.SetConnMaxIdleTime(5 * time.Minute)
	if err := db.Ping(); err != nil {
		return err
	}
	defaultDB = db
	return nil
}

func DB() *sql.DB { return defaultDB }

type User struct {
	ID           int64
	Email        string
	PasswordHash string
	Role         string
}

func GetUserByEmail(ctx context.Context, email string) (*User, error) {
	const q = `
		SELECT id, email, password_hash, COALESCE(role,'user')
		FROM users
		WHERE email = $1
		LIMIT 1
	`
	var u User
	err := DB().QueryRowContext(ctx, q, email).
		Scan(&u.ID, &u.Email, &u.PasswordHash, &u.Role)
	if err != nil {
		return nil, err
	}
	return &u, nil
}