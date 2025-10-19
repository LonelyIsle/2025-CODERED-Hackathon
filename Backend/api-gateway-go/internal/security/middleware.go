package security

import (
	"net"
	"net/http"
	"os"
	"strings"
	"sync"
	"time"
)

/* ---------- CSRF ---------- */

// CSRFMiddleware is a no-op when DISABLE_CSRF=true.
// Otherwise, it requires X-CSRF-Token on mutating methods.
func CSRFMiddleware(next http.Handler) http.Handler {
	if v := os.Getenv("DISABLE_CSRF"); v == "true" || v == "1" {
		return next
	}
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		switch r.Method {
		case http.MethodPost, http.MethodPut, http.MethodPatch, http.MethodDelete:
			if r.Header.Get("X-CSRF-Token") == "" {
				http.Error(w, "missing X-CSRF-Token", http.StatusForbidden)
				return
			}
		}
		next.ServeHTTP(w, r)
	})
}

// IssueCSRFToken returns a trivial CSRF token for the client to echo back.
// Our CSRFMiddleware only checks that the header is present, so any non-empty value works.
func IssueCSRFToken(w http.ResponseWriter, r *http.Request) {
	// If CSRF disabled, still respond OK so clients can proceed uniformly.
	w.Header().Set("X-CSRF-Token", "ok")
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write([]byte(`{"csrf":"ok"}`))
}

/* ---------- Rate limiting (token bucket per IP) ---------- */

type rlState struct {
	tokens     int
	lastRefill time.Time
}

type rateLimiter struct {
	mu          sync.Mutex
	perMinute   int
	maxBurst    int
	refillEvery time.Duration
	byIP        map[string]*rlState
}

// RateLimitMiddleware returns a Chi-compatible middleware.
// Example in main.go: r.Use(security.RateLimitMiddleware(120, time.Minute))
func RateLimitMiddleware(perMinute int, refillEvery time.Duration) func(http.Handler) http.Handler {
	if perMinute <= 0 {
		perMinute = 60
	}
	if refillEvery <= 0 {
		refillEvery = time.Minute
	}
	rl := &rateLimiter{
		perMinute:   perMinute,
		maxBurst:    perMinute, // allow bursting up to one minute worth
		refillEvery: refillEvery,
		byIP:        make(map[string]*rlState),
	}
	// Allow disabling via env for local testing
	if v := os.Getenv("DISABLE_RATELIMIT"); v == "true" || v == "1" {
		return func(next http.Handler) http.Handler { return next }
	}
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			ip := clientIP(r)
			if !rl.allow(ip) {
				http.Error(w, "rate limit exceeded", http.StatusTooManyRequests)
				return
			}
			next.ServeHTTP(w, r)
		})
	}
}

func (rl *rateLimiter) allow(ip string) bool {
	now := time.Now()
	rl.mu.Lock()
	defer rl.mu.Unlock()

	st := rl.byIP[ip]
	if st == nil {
		st = &rlState{tokens: rl.maxBurst, lastRefill: now}
		rl.byIP[ip] = st
	}

	// Refill based on elapsed intervals
	elapsed := now.Sub(st.lastRefill)
	if elapsed >= rl.refillEvery {
		chunks := int(elapsed / rl.refillEvery)
		st.tokens += chunks * rl.perMinute
		if st.tokens > rl.maxBurst {
			st.tokens = rl.maxBurst
		}
		st.lastRefill = st.lastRefill.Add(time.Duration(chunks) * rl.refillEvery)
	}

	if st.tokens <= 0 {
		return false
	}
	st.tokens--
	return true
}

func clientIP(r *http.Request) string {
	// Respect X-Forwarded-For (first entry)
	if xff := r.Header.Get("X-Forwarded-For"); xff != "" {
		parts := strings.Split(xff, ",")
		if len(parts) > 0 {
			return strings.TrimSpace(parts[0])
		}
	}
	// Fall back to RemoteAddr
	host, _, err := net.SplitHostPort(r.RemoteAddr)
	if err == nil && host != "" {
		return host
	}
	return r.RemoteAddr
}
