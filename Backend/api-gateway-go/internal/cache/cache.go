package cache

import (
	"context"
	"time"

	"github.com/redis/go-redis/v9"
)

var (
	rdb *redis.Client
	ctx = context.Background()
)

func Init(addr string, db int) {
	if addr == "" {
		addr = "127.0.0.1:6379"
	}
	rdb = redis.NewClient(&redis.Options{
		Addr: addr,
		DB:   db,
	})
}

func Set(key, val string, ttl time.Duration) error {
	return rdb.Set(ctx, key, val, ttl).Err()
}

func Get(key string) (string, error) {
	return rdb.Get(ctx, key).Result()
}

func Delete(key string) error {
	return rdb.Del(ctx, key).Err()
}