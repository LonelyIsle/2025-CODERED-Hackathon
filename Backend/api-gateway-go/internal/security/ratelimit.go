package security

import (
	"net"
	"net/http"
	"sync"
	"time"
)

type bucket struct {
	tokens int
	last   time.Time
}

func RateLimitMiddleware(limit int, per time.Duration) func(http.Handler) http.Handler {
	var mu sync.Mutex
	bk := map[string]*bucket{}
	refill := func(b *bucket) {
		now := time.Now()
		elapsed := now.Sub(b.last)
		if elapsed <= 0 {
			return
		}
		add := int(float64(limit) * (float64(elapsed) / float64(per)))
		if add > 0 {
			b.tokens += add
			if b.tokens > limit { b.tokens = limit }
			b.last = now
		}
	}
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			ip, _, _ := net.SplitHostPort(r.RemoteAddr)
			if ip == "" { ip = r.RemoteAddr }
			mu.Lock()
			b := bk[ip]
			if b == nil {
				b = &bucket{tokens: limit, last: time.Now()}
				bk[ip] = b
			}
			refill(b)
			if b.tokens <= 0 {
				mu.Unlock()
				http.Error(w, "rate limited", http.StatusTooManyRequests)
				return
			}
			b.tokens--
			mu.Unlock()
			next.ServeHTTP(w, r)
		})
	}
}