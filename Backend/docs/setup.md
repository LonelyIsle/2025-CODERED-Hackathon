# RHEL Setup
	1.	NVIDIA driver + CUDA
	•	Enable CUDA repo, install nvidia-driver + cuda-toolkit, reboot, nvidia-smi.
	2.	PostgreSQL 16 + pgvector
	•	Install PGDG packages postgresql16* + pgvector_16.
	•	Init DB, enable service; create user/db; CREATE EXTENSION vector;
	•	Run migrations 0001_init.sql, 0002_pgvector.sql, 0003_mitigations.sql.
	3.	Redis
	•	dnf install redis && systemctl enable --now redis.
	4.	Go / Rust / ONNX Runtime (GPU)
	•	dnf install golang; install Rust via rustup; unpack ONNX Runtime GPU build to /opt.
	•	Build services:
	•	C++ inference: cmake -B build ... && cmake --build build -j
	•	Rust worker: cargo build --release
	•	Go API: go build -o bin/climate-api ./cmd/server
	5.	Configuration
	•	Copy .env.example → .env:
```
PG_URL=postgres://climate:climate_pwd@127.0.0.1:5432/climate_db
REDIS_URL=redis://127.0.0.1:6379/0
INFERENCE_URL=http://127.0.0.1:5001
WORKER_URL=http://127.0.0.1:5002
LLM_URL=http://127.0.0.1:8081        # optional llama.cpp
CUDA_VISIBLE_DEVICES=0
```

	•	./services/inference-cpp/build/inference_server --port 5001
	•	./services/worker-rust/target/release/worker-rust --port 5002
	•	./api-gateway-go/bin/climate-api --port 8080
	•	(Optionally install systemd units from infra/systemd/*.)



# External exposure is ONLY NGINX on 443
BIND_HOST=127.0.0.1
API_PORT=8080

PG_URL=postgres://climate:climate_pwd@127.0.0.1:5432/climate_db
REDIS_URL=redis://127.0.0.1:6379/0

INFERENCE_URL=http://127.0.0.1:5001
WORKER_URL=http://127.0.0.1:5002
LLM_URL=http://127.0.0.1:8081          # optional, also bind 127.0.0.1

SESSION_SECRET=<random-32-bytes-hex>
PEPPER_SECRET=<random-32-bytes-hex>    # for password hashing
COOKIE_NAME=__Host-climate_sess
SESSION_TTL_HOURS=12
CSRF_HEADER=X-CSRF-Token

DISABLE_CORS=true