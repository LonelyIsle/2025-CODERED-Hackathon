package cache

import (
	"context"
	"log"
	"os"
	"time"

	"github.com/redis/go-redis/v9"
)

var Client *redis.Client
var ctx = context.Background()

func Init() {
	addr := os.Getenv("VALKEY_ADDR")
	if addr == "" {
		addr = "127.0.0.1:6379"
	}
	Client = redis.NewClient(&redis.Options{
		Addr: addr,
		DB:   0,
	})
	_, err := Client.Ping(ctx).Result()
	if err != nil {
		log.Fatalf("❌ Valkey connect fail: %v", err)
	}
	log.Println("✅ Connected to Valkey")
}

func Set(key string, val string, ttl time.Duration) {
	Client.Set(ctx, key, val, ttl)
}

func Get(key string) (string, error) {
	return Client.Get(ctx, key).Result()
}

func Delete(key string) {
	Client.Del(ctx, key)
}