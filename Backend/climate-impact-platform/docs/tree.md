climate-impact-platform/
├── api-gateway-go/
│   ├── cmd/server/main.go
│   ├── internal/
│   │   ├── auth/
│   │   │   ├── middleware.go        # session validate, role check
│   │   │   ├── handlers.go          # /api/auth/login, /logout, /me, csrf
│   │   │   └── password.go          # argon2id hashing/verify
│   │   ├── security/
│   │   │   ├── csrf.go              # CSRF token issue/verify
│   │   │   └── ratelimit.go         # per-IP/session limits
│   │   ├── handlers/                # /api/report/*, /api/admin/*
│   │   ├── clients/                 # inference/worker clients (loopback)
│   │   ├── db/                      # pgx repos incl. users, sessions
│   │   └── cache/redis.go
│   ├── web/                         # (optionally serve UI directly)
│   └── go.mod
│
├── services/inference-cpp/          # bind 127.0.0.1:5001
├── services/worker-rust/            # bind 127.0.0.1:5002
├── db/
│   ├── migrations/
│   │   ├── 0001_init.sql
│   │   ├── 0002_pgvector.sql
│   │   ├── 0003_mitigations.sql
│   │   └── 0004_auth.sql            # users, sessions tables
│   └── seed/
├── infra/
│   ├── nginx/site.conf              # above reverse proxy
│   ├── systemd/
│   │   ├── nginx.service            # packaged on RHEL
│   │   ├── climate-api.service      # ExecStart binds 127.0.0.1:8080
│   │   ├── inference.service        # 127.0.0.1:5001
│   │   └── worker.service           # 127.0.0.1:5002
│   └── tls/
└── docs/
    ├── SYSTEM_OVERVIEW.md
    ├── SECURITY_MODEL.md            # this section
    ├── API.md
    ├── DATA_MODEL.md
    └── SETUP_RHEL.md