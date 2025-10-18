climate-impact-platform/
├── api-gateway-go/                     # Go 1.22+ stateless API
│   ├── cmd/server/main.go
│   ├── internal/
│   │   ├── handlers/                   # /admin/*, /report/*
│   │   ├── services/                   # orchestrates recommender, inference
│   │   ├── clients/
│   │   │   ├── inference_client.go     # C++ service HTTP/gRPC
│   │   │   ├── worker_client.go        # Rust worker HTTP
│   │   │   └── llm_client.go           # optional llama.cpp
│   │   ├── db/                         # pgx repositories (companies, docs, passages,
│   │   │   ├── companies_repo.go       #  features_company, predictions, mitigations, recs)
│   │   │   └── ...
│   │   ├── cache/redis.go
│   │   └── recommender/                # fast rule+retrieval mapping risks→actions
│   │       ├── ranker.go
│   │       └── scoring.go
│   ├── go.mod / go.sum
│   └── Makefile
│
├── services/
│   ├── inference-cpp/                  # C++17 + ONNX Runtime (CUDA EP)
│   │   ├── CMakeLists.txt
│   │   ├── src/
│   │   │   ├── server.cpp              # /inference/score
│   │   │   ├── model_runner.cpp        # ONNX session, batching, fp16
│   │   │   ├── tokenizer.hpp/cpp       # HF tokenizers binding or custom BPE
│   │   │   └── utils.hpp/cpp
│   │   └── models/                     # exported fp16 ONNX (deberta-base multilabel)
│   └── worker-rust/                    # Rust 1.80 async worker
│       ├── src/
│       │   ├── main.rs                 # HTTP: /collect, /process
│       │   ├── ingest.rs               # scraping, robots.txt, rate limit
│       │   ├── parse.rs                # PDF→text, HTML→text, chunking
│       │   ├── embed.rs                # e5-base with onnxruntime-rs/tch-rs
│       │   ├── classify.rs             # multilabel classifier (onnx/tch)
│       │   ├── features.rs             # aggregation → features_company
│       │   └── jobs.rs                 # queue + progress
│       ├── Cargo.toml
│       └── Makefile
│
├── db/
│   ├── migrations/
│   │   ├── 0001_init.sql               # core tables
│   │   ├── 0002_pgvector.sql           # vector ext + indexes
│   │   └── 0003_mitigations.sql        # KB + recommendations
│   └── seed/
│       ├── example_companies.sql
│       └── mitigations_seed.sql        # initial KB actions
│
├── web/                                # simple UI (static or served by Go)
│   ├── admin.html
│   └── report.html
│
├── infra/
│   ├── systemd/
│   │   ├── climate-api.service
│   │   ├── inference.service
│   │   └── worker.service
│   ├── podman-compose.yml              # RHEL-friendly compose (with --gpus for inference)
│   └── tls/
│
├── docs/
│   ├── SYSTEM_OVERVIEW.md              # (this file)
│   ├── API.md                          # endpoint specs & payloads
│   ├── DATA_MODEL.md                   # schemas + indexes + ANN tuning
│   └── SETUP_RHEL.md                   # step-by-step install & runbook
│
├── .env.example
└── Makefile                            # top-level build targets



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