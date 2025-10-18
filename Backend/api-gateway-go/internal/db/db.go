package db

import (
	"context"
	"log"
	"os"

	"github.com/jackc/pgx/v5/pgxpool"
)

var Pool *pgxpool.Pool

func Init() {
	url := os.Getenv("PG_URL")
	if url == "" {
		log.Fatal("PG_URL not set")
	}
	cfg, err := pgxpool.ParseConfig(url)
	if err != nil {
		log.Fatalf("bad PG_URL: %v", err)
	}
	pool, err := pgxpool.NewWithConfig(context.Background(), cfg)
	if err != nil {
		log.Fatalf("db connect error: %v", err)
	}
	Pool = pool
	log.Println("✅ PostgreSQL connected")
}

func Close() {
	if Pool != nil {
		Pool.Close()
	}
}