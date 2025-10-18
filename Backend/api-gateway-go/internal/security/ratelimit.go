package security

import (
	"net"
	"net/http"
	"sync"
	"time"
)

var visitors = make(map[string]time.Time)
var mu sync.Mutex

func RateLimitMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		ip, _, _ := net.SplitHostPort(r.RemoteAddr)
		mu.Lock()
		last, ok := visitors[ip]
		if ok && time.Since(last) < 200*time.Millisecond {
			mu.Unlock()
			http.Error(w, "too many requests", http.StatusTooManyRequests)
			return
		}
		visitors[ip] = time.Now()
		mu.Unlock()
		next.ServeHTTP(w, r)
	})
}